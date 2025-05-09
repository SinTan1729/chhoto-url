# SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
# SPDX-License-Identifier: MIT

# .env file has the variables $DOCKER_USERNAME and $PASSWORD defined
include .env

setup:
	cargo install cross
	rustup target add x86_64-unknown-linux-musl
	docker buildx create --use --platform=linux/arm64,linux/amd64 --name multi-platform-builder
	docker buildx inspect --bootstrap

build-dev:
	cargo build --release --locked --manifest-path=actix/Cargo.toml --target x86_64-unknown-linux-musl

docker-local: build-dev
	docker build --tag chhoto-url --build-arg TARGETARCH=amd64 -f Dockerfile.multiarch .

docker-stop:
	docker ps -q --filter "name=chhoto-url" | xargs -r docker stop
	docker ps -aq --filter "name=chhoto-url" | xargs -r docker rm

docker-test: docker-local docker-stop
	docker run -t -p ${port}:${port} --name chhoto-url --env-file ./.env -v "${db_file}:${db_url}" -d chhoto-url
	docker logs chhoto-url -f 

docker-dev: build-dev
	docker build --push --tag ${docker_username}/chhoto-url:dev --build-arg TARGETARCH=amd64 -f Dockerfile.multiarch .

build-release:
	cross build --release --locked --manifest-path=actix/Cargo.toml --target aarch64-unknown-linux-musl
	cross build --release --locked --manifest-path=actix/Cargo.toml --target armv7-unknown-linux-musleabihf
	cross build --release --locked --manifest-path=actix/Cargo.toml --target x86_64-unknown-linux-musl

v_patch := $(shell cat actix/Cargo.toml | sed -rn 's/^version = "(.+)"$$/\1/p')
v_minor := $(shell cat actix/Cargo.toml | sed -rn 's/^version = "(.+)\..+"$$/\1/p')
v_major := $(shell cat actix/Cargo.toml | sed -rn 's/^version = "(.+)\..+\..+"$$/\1/p')
docker-release: build-release
	docker buildx build --push --tag ${docker_username}/chhoto-url:${v_major} --tag ${docker_username}/chhoto-url:${v_minor} \
		--tag ${docker_username}/chhoto-url:${v_patch} --tag ${docker_username}/chhoto-url:latest \
		--platform linux/amd64,linux/arm64,linux/arm/v7 -f Dockerfile.multiarch .

clean:
	docker ps -q --filter "name=chhoto-url" | xargs -r docker stop
	docker ps -aq --filter "name=chhoto-url" | xargs -r docker rm
	cargo clean --manifest-path=actix/Cargo.toml

.PHONY: build-dev docker-local docker-stop build-release
