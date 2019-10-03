###################
### BUILD STAGE ###
###################
# TODO: use rust stable branch
FROM rustlang/rust:nightly as build_stage

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
ENV DATABASE_URL="postgres://postgres@database/laggit"

# Install dependencies
RUN apt-get update \
 && apt-get install -y libpq5 \
 && apt-get autoremove && apt-get autoclean

# Copy application binary
COPY --from=build_stage /app/target/release/laggit_backend /usr/local/bin/

# Copy database migrations
COPY backend/migrations /migrations

CMD /usr/local/bin/laggit_backend
