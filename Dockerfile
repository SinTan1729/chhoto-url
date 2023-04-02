FROM rust:1 as build

RUN mkdir /actix
WORKDIR /actix

COPY ./actix/cargo.toml /actix/cargo.toml
COPY ./actix/cargo.lock /actix/cargo.lock

RUN cargo build --deps-only

COPY ./actix /actix

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian10

EXPOSE 2000

COPY --from=build /usr/local/cargo/bin/actix /usr/local/bin/actix

CMD ["actix"]
