###################
### BUILD STAGE ###
###################
FROM rust:1.45 as build_stage

# Install build dependencies
RUN rustup update

# Build project
WORKDIR /app
COPY . .
WORKDIR /app/backend
RUN cargo build --release

########################
### PRODUCTION STAGE ###
########################
FROM debian:stable-slim

# Default to staging environment
# Override with "production" when deploying for real
ENV ROCKET_ENV="staging"

# Default to always running database migrations
ENV RUN_MIGRATIONS="true"

# Basic default configuration for database, suitable for dev.
ENV DATABASE_URL="postgres://postgres@database/strecklistan"

# Install dependencies
RUN apt-get update \
 && apt-get install -y libpq5 openssl \
 && apt-get autoremove && apt-get autoclean

# Copy application binary
COPY --from=build_stage /app/target/release/strecklistan_backend /usr/local/bin/

# Copy database migrations
COPY backend/migrations /migrations

CMD /usr/local/bin/strecklistan_backend
