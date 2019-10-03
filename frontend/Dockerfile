FROM rustlang/rust:nightly

LABEL maintainer="Joakim Hulthe <joakim@hulthe.net>"

# # Remove this when nginx adds webassembly as a mimetype
# RUN sed -i "/types {/a application/wasm  wasm;" /etc/nginx/mime.types

# RUN rm /etc/nginx/conf.d/*
# COPY nginx.conf /etc/nginx/conf.d/custom.conf

# Install build dependencies
# RUN rustup update

WORKDIR /app/frontend

RUN ls -la

RUN rustup update
RUN cargo install cargo-make

CMD $(cargo make serve &) && cargo make watch
