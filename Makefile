setup:
	cargo install cross
	docker buildx create --use --platform=linux/arm64,linux/amd64 --name multi-platform-builder
	docker buildx inspect --bootstrap

build:
	cross build --release --locked --manifest-path=actix/Cargo.toml --target aarch64-unknown-linux-musl
	cross build --release --locked --manifest-path=actix/Cargo.toml --target armv7-unknown-linux-musleabihf
	cross build --release --locked --manifest-path=actix/Cargo.toml --target x86_64-unknown-linux-musl

docker: build
	docker buildx build --push --tag sintan1729/chhoto-url:dev --platform linux/amd64,linux/arm64,linux/arm/v7 -f Dockerfile.multiarch .

clean:
	cargo clean --manifest-path=actix/Cargo.toml
	rm -rf .docker

.PHONY: build
