FROM rust:1.85.0-alpine



WORKDIR /app
COPY . .

RUN cargo build --release
