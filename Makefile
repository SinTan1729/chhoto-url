# SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
# SPDX-License-Identifier: MIT

include .env

.PHONY: clean test setup build podman-build podman-stop podman-run podman-test build-release tag audit merge

setup:
	rustup target add x86_64-unknown-linux-musl
	podman buildx inspect --bootstrap

merge:
	git switch main
	git merge dev
	git switch dev

short_sha := $(shell git rev-parse --short HEAD) 
build:
	CARGO_GIT_COMMIT=${short_sha} cargo build --release --locked --manifest-path=actix/Cargo.toml --target x86_64-unknown-linux-musl

podman-build: build
	podman build --tag chhoto-url --build-arg TARGETARCH=amd64 -f deploy/Dockerfile.alpine .

podman-stop:
	podman ps -q --filter "name=chhoto-url" | xargs -r podman stop
	podman ps -aq --filter "name=chhoto-url" | xargs -r podman rm

test: audit
	cargo test --release --locked --manifest-path=actix/Cargo.toml --target x86_64-unknown-linux-musl

audit:
	cargo audit --file actix/Cargo.lock

podman-run: podman-stop
	podman run -t -p ${CHHOTO_LISTEN_PORT}:${CHHOTO_LISTEN_PORT} --name chhoto-url --env-file ./.env -v "${DB_DIR}:/data" -d chhoto-url
	podman logs chhoto-url -f 

podman-test: test podman-build podman-run

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

