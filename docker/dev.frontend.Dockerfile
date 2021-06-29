FROM rust:1.53 as build_stage

RUN cargo install -f cargo-make

VOLUME /app
VOLUME /app/frontend/pkg

ENV CARGO_BUILD_TARGET_DIR /target
VOLUME /target

WORKDIR /app/frontend
CMD cargo make watch
