# SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
# SPDX-License-Identifier: MIT

include .env

.PHONY: clean test setup build podman-build podman-stop podman-test build-release tag audit

setup:
	rustup target add x86_64-unknown-linux-musl
	podman buildx inspect --bootstrap

build:
	cargo build --release --locked --manifest-path=actix/Cargo.toml --target x86_64-unknown-linux-musl

podman-build: build
	podman build --tag chhoto-url --build-arg TARGETARCH=amd64 -f Dockerfile.alpine .

podman-stop:
	podman ps -q --filter "name=chhoto-url" | xargs -r podman stop
	podman ps -aq --filter "name=chhoto-url" | xargs -r podman rm

test: audit
	cargo test --release --locked --manifest-path=actix/Cargo.toml --target x86_64-unknown-linux-musl

audit:
	cargo audit --file actix/Cargo.lock

podman-test: test podman-build podman-stop
	podman run -t -p ${port}:${port} --name chhoto-url --env-file ./.env -v "${db_dir}:/data" -d chhoto-url
	podman logs chhoto-url -f 

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

clean: podman-stop
	cargo clean --manifest-path=actix/Cargo.toml

