###################
### BUILD STAGE ###
###################
FROM rust:1.49 as build_stage

# Install build dependencies
#RUN rustup update
RUN cargo install -f cargo-make

# Build project
WORKDIR /app
COPY . .

WORKDIR /app/backend
RUN cargo build --release

WORKDIR /app/frontend
RUN cargo make build_release

########################
### PRODUCTION STAGE ###
########################
FROM debian:stable-slim

# Default to staging environment
# Override with "production" when deploying for real
ENV ROCKET_ENV="staging"

ENV ROCKET_ADDRESS="0.0.0.0"
ENV ROCKET_PORT="8000"

# Default to always running database migrations
ENV RUN_MIGRATIONS="true"

# Basic default configuration for database, suitable for dev.
ENV DATABASE_URL="postgres://postgres@database/strecklistan"

# Install dependencies
RUN apt-get update \
 && apt-get install -y libpq5 openssl \
 && apt-get autoremove && apt-get autoclean

RUN mkdir -p /www
WORKDIR /

# Copy application binary
COPY --from=build_stage /app/target/release/strecklistan_backend /usr/local/bin/

# Copy static web files
COPY --from=build_stage /app/frontend/pkg    /www/pkg
COPY --from=build_stage /app/frontend/static /www/static
COPY backend/www/index.html               /www/index.html

# Copy database migrations
COPY backend/migrations /migrations

CMD /usr/local/bin/strecklistan_backend
