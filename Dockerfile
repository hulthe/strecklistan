###################
### BUILD STAGE ###
###################
FROM rust:1.53 as build_stage

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

# Rocket web server configuration
ENV ROCKET_ADDRESS="0.0.0.0"
ENV ROCKET_PORT="8000"

# Default to always running database migrations
ENV RUN_MIGRATIONS="true"

# Basic default configuration for database, suitable for dev.
ENV DATABASE_URL="postgres://postgres@database/strecklistan"

# Enable cache-control
ENV ENABLE_STATIC_FILE_CACHE="true"
ENV STATIC_FILES_MAX_AGE="0"

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

# Copy database migrations
COPY backend/migrations /migrations

CMD /usr/local/bin/strecklistan_backend
