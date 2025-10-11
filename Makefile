# SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
# SPDX-License-Identifier: MIT

# .env file has the variables $podman_USERNAME and $PASSWORD defined
include .env

.PHONY: clean test setup build-dev podman-local podman-stop podman-test build-release tag audit

setup:
	# cargo install cross
	rustup target add x86_64-unknown-linux-musl
	# podman buildx create --use --platform=linux/arm64,linux/amd64,linux/arm/v7 --name multi-platform-builder
	podman buildx inspect --bootstrap

build-dev:
	cargo build --release --locked --manifest-path=actix/Cargo.toml --target x86_64-unknown-linux-musl

podman-local: build-dev
	podman build --tag chhoto-url --build-arg TARGETARCH=amd64 -f Dockerfile.alpine .

podman-stop:
	podman ps -q --filter "name=chhoto-url" | xargs -r podman stop
	podman ps -aq --filter "name=chhoto-url" | xargs -r podman rm

test: audit
	cargo test --release --locked --manifest-path=actix/Cargo.toml --target x86_64-unknown-linux-musl

audit:
	cargo audit --file actix/Cargo.lock

podman-test: podman-local podman-stop test
	podman run -t -p ${port}:${port} --name chhoto-url --env-file ./.env -v "${db_dir}:/data" -d chhoto-url
	podman logs chhoto-url -f 

# podman-dev: test build-dev
# 	podman build --push --tag ghcr.io/${github_username}/chhoto-url:dev --build-arg TARGETARCH=amd64 -f Dockerfile.alpine .

# build-release: test
# 	cross build --release --locked --manifest-path=actix/Cargo.toml --target aarch64-unknown-linux-musl
# 	cross build --release --locked --manifest-path=actix/Cargo.toml --target armv7-unknown-linux-musleabihf
# 	cross build --release --locked --manifest-path=actix/Cargo.toml --target x86_64-unknown-linux-musl

conf_tag := $(shell cat actix/Cargo.toml | sed -rn 's/^version = "(.+)"$$/\1/p')
last_tag := $(shell git tag -l | tail -1)
bumped := $(shell git log -1 --pretty=%B | grep "build: Bumped version to " | wc -l)
uncommitted := $(shell git status --porcelain=v1 2>/dev/null | wc -l)
tag:
ifeq (${bumped}, 1)
ifneq (${uncommitted}, 0)
	false;
endif
ifneq (${conf_tag}, ${last_tag})
	git tag ${conf_tag} -m "Version ${conf_tag}"
endif
else
	false;
endif

# v_patch := $(shell cat actix/Cargo.toml | sed -rn 's/^version = "(.+)"$$/\1/p')
# v_minor := $(shell cat actix/Cargo.toml | sed -rn 's/^version = "(.+)\..+"$$/\1/p')
# v_major := $(shell cat actix/Cargo.toml | sed -rn 's/^version = "(.+)\..+\..+"$$/\1/p')
# podman-release: tag build-release
#	minify -rsi resources/
# 	podman buildx build --push --tag ${podman_username}/chhoto-url:${v_major} --tag ${podman_username}/chhoto-url:${v_minor} \
# 		--tag ${podman_username}/chhoto-url:${v_patch} --tag ${podman_username}/chhoto-url:latest \
# 		--platform linux/amd64,linux/arm64,linux/arm/v7 -f Dockerfile.alpine .
# 	podman buildx build --push --tag ghcr.io/${github_username}/chhoto-url:${v_major} --tag ghcr.io/${github_username}/chhoto-url:${v_minor} \
# 		--tag ghcr.io/${github_username}/chhoto-url:${v_patch} --tag ghcr.io/${github_username}/chhoto-url:latest \
# 		--platform linux/amd64,linux/arm64,linux/arm/v7 -f Dockerfile.scratch .
#	git restore resources/

clean:
	podman ps -q --filter "name=chhoto-url" | xargs -r podman stop
	podman ps -aq --filter "name=chhoto-url" | xargs -r podman rm
	cargo clean --manifest-path=actix/Cargo.toml

