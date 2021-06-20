FROM rust:1.53 as build_stage

RUN apt-get update &&\
    apt-get install -y postgresql-client netcat &&\
    apt-get autoremove && apt-get autoclean

RUN cargo install -f cargo-watch diesel_cli


VOLUME /app
VOLUME /app/frontend/pkg

ENV CARGO_BUILD_TARGET_DIR /target
VOLUME /target

ENV ROCKET_ADDRESS 0.0.0.0

WORKDIR /app/backend
CMD sh /app/docker/start_backend.sh

