run:
	cargo run

bench:
	cargo build --release
	./scripts/benchmark.sh