default:
	@just --list
run:
	cargo run -- \
		--filter filter/ \
		--source submissions.zip \
		--target out/
clear:
	clear
build:
	cargo build
clean:
	cargo clean
release:
	cargo build --release
install: clear build release
	cp target/debug/unpack_moodle ~/.local/bin/unpack_moodle_dev
	cp target/release/unpack_moodle ~/.local/bin/unpack_moodle
