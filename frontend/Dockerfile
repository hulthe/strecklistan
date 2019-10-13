FROM rustlang/rust:nightly

LABEL maintainer="Joakim Hulthe <joakim@hulthe.net>"

WORKDIR /app/frontend

RUN ls -la

RUN rustup update
RUN cargo install cargo-make

CMD $(cargo make serve &) && cargo make watch
