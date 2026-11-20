run:
	cargo run

bench:
	cargo build --release
	./scripts/benchmark.sh

build-and-copy:
	cargo build --release
	cp target/release/ml4w-killactive ~/.config/hypr/scripts/ml4w-killactive
