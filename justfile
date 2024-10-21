target := "x86_64-unknown-linux-musl"

default:
	@just --list
run:
	cargo run -- \
		--filter filter/ \
		--source submissions.zip \
		--target out/
build:
	cargo build
clean:
	cargo clean
release:
	cargo build --release
	cargo build --release --target {{target}}

