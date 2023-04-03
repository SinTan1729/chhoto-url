FROM rust:1 as build
RUN cargo install cargo-build-deps

RUN cargo new --bin simply-shorten
WORKDIR /simply-shorten

COPY ./actix/Cargo.toml .
COPY ./actix/Cargo.lock .

RUN cargo build-deps --release

COPY ./actix/src ./src
COPY ./actix/resources ./resources

RUN cargo build --release

FROM gcr.io/distroless/cc-debian10

EXPOSE 2000

WORKDIR /opt

COPY --from=build /simply-shorten/target/release/simply-shorten /opt/simply-shorten
COPY --from=build /simply-shorten/resources /opt/resources
COPY ./urls.sqlite /opt/urls.sqlite

CMD ["./simply-shorten"]
