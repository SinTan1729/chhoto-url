# SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
# SPDX-License-Identifier: MIT

include .env

.PHONY: clean test setup build podman-build podman-build-release podman-stop podman-run podman-test podman-test-release build-release tag upgrade-deps-pre upgrade-deps merge reset db

setup:
	rustup target add x86_64-unknown-linux-musl
	podman buildx inspect --bootstrap

merge:
	git switch main
	git merge dev
	git switch dev

short_sha := $(shell git rev-parse --short HEAD) 
build:
	CARGO_GIT_COMMIT=${short_sha} cargo build --locked --manifest-path=backend/Cargo.toml --target x86_64-unknown-linux-musl
build-release:
	CARGO_GIT_COMMIT=${short_sha} cargo build --release --locked --manifest-path=backend/Cargo.toml --target x86_64-unknown-linux-musl
test:
	cargo audit --file backend/Cargo.lock
	CARGO_GIT_COMMIT=${short_sha} cargo test --locked --manifest-path=backend/Cargo.toml --target x86_64-unknown-linux-musl

podman-clean:
	podman image prune -f
podman-build: build
	podman build --tag chhoto-url:debug -f deploy/Containerfile.debug .
podman-build-release: build-release
	podman build --tag chhoto-url:release --build-arg TARGETARCH=amd64 -f deploy/Containerfile.alpine .

podman-stop:
	podman ps -q --filter "name=chhoto-url" | xargs -r podman stop
	podman ps -aq --filter "name=chhoto-url" | xargs -r podman rm

podman-run: podman-stop
	podman run -t -p ${CHHOTO_LISTEN_PORT}:${CHHOTO_LISTEN_PORT} --name chhoto-url --env-file ./.env -v "${DB_DIR}:/data" -d chhoto-url:debug
	podman logs chhoto-url -f 
podman-run-release: podman-stop
	podman run -t -p ${CHHOTO_LISTEN_PORT}:${CHHOTO_LISTEN_PORT} --name chhoto-url --env-file ./.env -v "${DB_DIR}:/data" -d chhoto-url:release
	podman logs chhoto-url -f 

reset-db: podman-stop
	rm -f testing-data/urls.sqlite-shm testing-data/urls.sqlite-wal
	cp testing-data/urls1.sqlite testing-data/urls.sqlite
podman-test: test podman-build podman-clean podman-run
podman-test-release: test podman-build-release podman-clean podman-run-release

upgrade-deps-pre:
	cargo upgrade --manifest-path=backend/Cargo.toml --verbose
	cargo update --manifest-path=backend/Cargo.toml --verbose
	git add backend/Cargo.{toml,lock}
	git commit -m "chore: Updated deps"
upgrade-deps: upgrade-deps-pre test

conf_tag := $(shell cat backend/Cargo.toml | sed -rn 's/^version = "(.+)"$$/\1/p')
last_tag := $(shell git tag -l | tail -1)
bumped := $(shell git log -1 --pretty=%B | grep "build: Bumped version to " | wc -l)
uncommitted := $(shell git status --porcelain=v1 2>/dev/null | wc -l)
upgrade-version:
	cargo update --manifest-path=backend/Cargo.toml -p chhoto-url --verbose
	git add backend/Cargo.{toml,lock}
	git commit -m "build: Bumped version to ${conf_tag}"

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
	cargo clean --manifest-path=backend/Cargo.toml

