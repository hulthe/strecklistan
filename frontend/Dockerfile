FROM nginx:alpine

LABEL maintainer="Joakim Hulthe <joakim@hulthe.net>"

# Remove this when nginx adds webassembly as a mimetype
RUN sed -i "/types {/a application/wasm  wasm;" /etc/nginx/mime.types

RUN rm /etc/nginx/conf.d/*
COPY prod.nginx.conf /etc/nginx/conf.d/custom.conf

COPY index.html /usr/share/nginx/html/index.html
COPY pkg /usr/share/nginx/html/pkg
COPY static /usr/share/nginx/html/static
