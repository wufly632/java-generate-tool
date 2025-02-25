FROM rust:1.85.0-alpine

RUN apk add --no-cache musl-dev

# 添加 musl 目标
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app
COPY . .

RUN cargo build --release --target=x86_64-unknown-linux-musl
