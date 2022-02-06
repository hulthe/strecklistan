FROM rust:1.58.1 as build_stage

RUN cargo install --locked cargo-make trunk
RUN rustup target add wasm32-unknown-unknown

VOLUME /out
ENV CARGO_BUILD_TARGET_DIR /out/target
ENV TRUNK_DIST_DIR /out/dist

VOLUME /app
WORKDIR /app/frontend
CMD trunk serve
