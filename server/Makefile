.PHONY: watchtest build

watchtest:
	RUST_BACKTRACE=0 cargo watch -x test -i Makefile

test:
	RUST_LIB_BACKTRACE=0 RUST_BACKTRACE=0 cargo test -- --nocapture --color always

build:
	cargo build

build-release:
	cargo build --release