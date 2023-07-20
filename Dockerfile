FROM rust:latest
LABEL org.opencontainers.image.source="https://github.com/its4u/cert-manager-routes-controller"

WORKDIR /app
COPY ctrl .
RUN cargo build
CMD cargo run
