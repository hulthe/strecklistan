FROM rustlang/rust:nightly

RUN rustup update
RUN cargo install cargo-watch

WORKDIR /app/backend

COPY .env .env

VOLUME ./cargocache
CMD cargo watch -x run
