FROM rust:1.68.2-alpine3.17

RUN apk add g++ make python3 git

WORKDIR /app
COPY . .

RUN cargo build --release -q