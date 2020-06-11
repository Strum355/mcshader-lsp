.PHONY: watchtest build

watchtest:
	RUST_BACKTRACE=0 cargo watch -x test -i Makefile

build:
	cargo build --release