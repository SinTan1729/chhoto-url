FROM rust:1 as build
RUN cargo install cargo-build-deps

RUN cargo new --bin simply-shorten
WORKDIR /simply-shorten

COPY ./actix/Cargo.toml .
COPY ./actix/Cargo.lock .

RUN cargo build-deps --release

COPY ./actix/src ./src

RUN cargo build --release

FROM frolvlad/alpine-glibc:latest

RUN apk add sqlite-libs

WORKDIR /opt

COPY --from=build /simply-shorten/target/release/simply-shorten /opt/simply-shorten
COPY ./actix/resources /opt/resources

CMD ["./simply-shorten"]
