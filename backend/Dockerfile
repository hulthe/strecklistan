###################
### BUILD STAGE ###
###################
# TODO: use rust stable branch
FROM rustlang/rust:nightly as build_stage

# Install build dependencies
RUN rustup update

# Build diesel_cli with only postgres support
RUN cargo install diesel_cli --no-default-features --features "postgres"

# Build project
WORKDIR /app
COPY . .
RUN cargo build --release

########################
### PRODUCTION STAGE ###
########################
FROM debian:stable-slim

# Default to staging environment
# Override with "production" when deploying for real
ENV ROCKET_ENV="staging"

# Basic default configuration for database, suitable for dev.
ENV DATABASE_URL="postgres://postgres@database/laggit"

# Install dependencies
RUN apt-get update \
 && apt-get install -y libpq5 \
 && apt-get autoremove && apt-get autoclean

# Copy diesel_cli
COPY --from=build_stage /usr/local/cargo/bin/diesel /usr/local/bin/diesel_cli

# Copy application binary
COPY --from=build_stage /app/target/release/laggit_backend /usr/local/bin/

CMD /usr/local/bin/laggit_backend
