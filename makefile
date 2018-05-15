build-fixture:
	make -C tests/fixtures/sandbox build

dev-test:
	RUST_LOG="armv4=debug" cargo test

test-watch:
	cargo watch -x test	

test:
	cargo test	

build: 
	cargo +nightly build --features clippy