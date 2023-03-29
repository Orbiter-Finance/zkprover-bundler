FROM rust:1.68.2-alpine3.17

WORKDIR /app
COPY . .

RUN cargo build --release -q