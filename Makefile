setup:
	cargo install cross
	docker buildx create --use --platform=linux/arm64,linux/amd64 --name multi-platform-builder
	docker buildx inspect --bootstrap

build:
	cross build --release --locked --manifest-path=actix/Cargo.toml --target aarch64-unknown-linux-musl
	cross build --release --locked --manifest-path=actix/Cargo.toml --target x86_64-unknown-linux-musl

docker: build
	mkdir -p .docker/amd64 .docker/arm64
	cp actix/target/aarch64-unknown-linux-musl/release/chhoto-url .docker/arm64/
	cp actix/target/x86_64-unknown-linux-musl/release/chhoto-url .docker/amd64/
	docker buildx build --push --tag sintan1729/chhoto-url:dev --platform linux/amd64,linux/arm64 .

clean:
	cargo clean --manifest-path=actix/Cargo.toml
	rm -rf .docker

.PHONY: build clean docker
