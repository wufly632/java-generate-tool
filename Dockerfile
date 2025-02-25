FROM rust:1.85.0

# RUN apk add --no-cache musl-dev openssl-dev gcc

# 安装 cargo-generate
RUN cargo install cargo-generate

# 添加 musl 目标
# RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app
COPY . .

RUN mkdir -p output
RUN cargo build --release
