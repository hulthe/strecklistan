FROM rustlang/rust:nightly

RUN rustup update
RUN cargo install cargo-watch

WORKDIR /app/backend

CMD cargo watch -x run
