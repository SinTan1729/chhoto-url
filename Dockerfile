FROM rust:slim AS build
ENV TARGET x86_64-unknown-linux-musl

RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add "$TARGET"

RUN cargo install cargo-build-deps

RUN cargo new --bin simply-shorten
WORKDIR /simply-shorten

RUN rustup target add x86_64-unknown-linux-musl

COPY ./actix/Cargo.toml .
COPY ./actix/Cargo.lock .

RUN cargo build-deps --release --target=x86_64-unknown-linux-musl

COPY ./actix/src ./src

RUN cargo build --release --locked --target "$TARGET"

FROM scratch

COPY --from=build /simply-shorten/target/x86_64-unknown-linux-musl/release/simply-shorten /simply-shorten
COPY ./actix/resources /resources

CMD ["/simply-shorten"]
