# killactive — dev shortcuts
# Override destination: make install PREFIX=$HOME/.config/hypr/scripts

PREFIX         ?= $(HOME)/.local/bin
BIN            := killactive
RELEASE        := target/release/$(BIN)
STATIC_RELEASE := target/x86_64-unknown-linux-musl/release/$(BIN)

.PHONY: build build-static install install-static uninstall bench bench-static test fmt lint clean help
.DEFAULT_GOAL := build

build:
	cargo build --release

build-static:
	cargo build --target=x86_64-unknown-linux-musl --release

install: build
	install -Dm755 $(RELEASE) $(PREFIX)/$(BIN)

install-static: build-static
	install -Dm755 $(STATIC_RELEASE) $(PREFIX)/$(BIN)

uninstall:
	rm -f $(PREFIX)/$(BIN)

bench: build
	./scripts/benchmark.sh

bench-static: build build-static
	./scripts/benchmark.sh --static

test:
	cargo test

fmt:
	cargo fmt

lint:
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean

help:
	@echo "build        - cargo build --release"
	@echo "build-static - cargo build --target=x86_64-unknown-linux-musl --release"
	@echo "install      - copy binary to PREFIX [$(PREFIX)]"
	@echo "install-static - copy musl static binary to PREFIX [$(PREFIX)]"
	@echo "uninstall    - remove it"
	@echo "bench        - build + hyperfine comparison"
	@echo "bench-static - build + musl build + hyperfine comparison"
	@echo "test         - cargo test"
	@echo "fmt          - cargo fmt"
	@echo "lint         - cargo clippy -D warnings"
	@echo "clean        - cargo clean"
