FROM rust:1.52-alpine AS build
WORKDIR /build

# https://benjamincongdon.me/blog/2019/12/04/Fast-Rust-Docker-Builds-with-cargo-vendor/
COPY Cargo.lock .
COPY Cargo.toml .
COPY src/main.rs ./src/main.rs
RUN mkdir .cargo && \
    cargo vendor > .cargo/config

RUN apk add --no-cache musl-dev

COPY . .
RUN cargo build --release && \
    cargo install --path . --verbose



FROM scratch
WORKDIR /app

COPY --from=build /build/target/release/narc /app/narc

ENTRYPOINT ["/app/narc"]
