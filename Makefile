setup:
	cargo install cross

build:
	cross build --release --locked --manifest-path=actix/Cargo.toml --target aarch64-unknown-linux-musl
	cross build --release --locked --manifest-path=actix/Cargo.toml --target x86_64-unknown-linux-musl

docker: build
	mkdir -p linux/amd64 linux/arm64
	cp actix/target/aarch64-unknown-linux-musl/release/chhoto-url linux/aarch64/
	cp actix/target/x86_64-unknown-linux-musl/release/chhoto-url linux/amd64/
	docker buildx -t chhoto-url --platform linux/amd64,linux/arm64,linux/arm/v7 .
	rm -rf linux

clean:
	cargo clean --manifest-path=actix/Cargo.toml

.PHONY: build clean docker
