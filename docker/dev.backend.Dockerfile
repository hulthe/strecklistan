FROM rust:1.58.1 as build_stage

RUN apt-get update &&\
    apt-get install -y postgresql-client netcat &&\
    apt-get autoremove && apt-get autoclean

RUN cargo install --locked cargo-watch diesel_cli

VOLUME /out
ENV CARGO_BUILD_TARGET_DIR /out/target

VOLUME /app
WORKDIR /app/backend
ENV ROCKET_ADDRESS 0.0.0.0
CMD sh /app/docker/start_backend.sh

