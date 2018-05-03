build-fixture:
	make -C tests/fixtures/simple build

dev-test:
	RUST_LOG="armv4=debug" cargo test

test:
	cargo test	