# killactive — dev shortcuts
# Override destination: make install PREFIX=$HOME/.config/hypr/scripts

PREFIX  ?= $(HOME)/.local/bin
BIN     := killactive
RELEASE := target/release/$(BIN)

.PHONY: build install uninstall bench test fmt lint clean help
.DEFAULT_GOAL := build

build:
	cargo build --release

install: build
	install -Dm755 $(RELEASE) $(PREFIX)/$(BIN)

uninstall:
	rm -f $(PREFIX)/$(BIN)

bench: build
	./scripts/benchmark.sh

test:
	cargo test

fmt:
	cargo fmt

lint:
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean

help:
	@echo "build     - cargo build --release"
	@echo "install   - copy binary to PREFIX [$(PREFIX)]"
	@echo "uninstall - remove it"
	@echo "bench     - build + hyperfine comparison"
	@echo "test      - cargo test"
	@echo "fmt       - cargo fmt"
	@echo "lint      - cargo clippy -D warnings"
	@echo "clean     - cargo clean"