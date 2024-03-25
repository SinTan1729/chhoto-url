FROM lukemathwalker/cargo-chef:latest-rust-slim AS chef
WORKDIR /chhoto-url

FROM chef as planner
COPY ./actix/Cargo.toml ./actix/Cargo.lock .
COPY ./actix/src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl

COPY --from=planner /chhoto-url/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer
RUN cargo chef cook --release --target=x86_64-unknown-linux-musl --recipe-path recipe.json

COPY ./actix/Cargo.toml ./actix/Cargo.lock .
COPY ./actix/src ./src
# Build application
RUN cargo build --release --target=x86_64-unknown-linux-musl --locked --bin chhoto-url

FROM scratch
COPY --from=builder /chhoto-url/target/x86_64-unknown-linux-musl/release/chhoto-url /chhoto-url
COPY ./resources /resources
ENTRYPOINT ["/chhoto-url"]

