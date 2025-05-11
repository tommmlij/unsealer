FROM rust:alpine AS builder
WORKDIR /app

RUN apk add --no-cache musl-dev build-base libsodium-dev

RUN rustup target add x86_64-unknown-linux-musl

# Copy only necessary files first to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo fetch --locked


COPY src ./src

RUN cargo build --release --target x86_64-unknown-linux-musl
RUN ls -lisaR


FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/unsealer /opt/unsealer/

ENTRYPOINT ["/opt/unsealer/unsealer"]