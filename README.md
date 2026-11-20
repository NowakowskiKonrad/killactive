# ml4w-killactive

Quick little program to make closing apps and panels in ML4W dotfiles instant.

The original shell script has a noticeable lag (~115 ms) from spawning multiple `qs`, `hyprctl`, and `jq` processes. This utility talks directly to Quickshell (`QDataStream`) and Hyprland over UNIX sockets via IPC.

## Benchmark

```text
Benchmark 1: Bash (original)
  Time (mean ± σ):     109.7 ms ±   1.0 ms    [User: 77.4 ms, System: 33.4 ms]
  Range (min … max):   107.4 ms … 112.1 ms    50 runs
 
Benchmark 2: Bash (fast)
  Time (mean ± σ):      41.8 ms ±   0.4 ms    [User: 75.6 ms, System: 33.3 ms]
  Range (min … max):    40.9 ms …  42.8 ms    50 runs
 
Benchmark 3: Rust (binary)
  Time (mean ± σ):     855.0 µs ± 106.9 µs    [User: 303.7 µs, System: 439.6 µs]
  Range (min … max):   638.7 µs … 991.3 µs    50 runs
 
Summary
  Rust (binary) ran
   48.90 ± 6.13 times faster than Bash (fast)
  128.25 ± 16.08 times faster than Bash (original)

```
*Disclaimer: `scripts/killactive_fast.sh` was written entirely by AI as a quick proof-of-concept to test parallel Bash overhead before rewriting the utility in Rust.*

## Installation & Usage

1. Build and copy the binary:

```bash
cargo build --release
cp target/release/killactive_rs ~/.local/bin/ml4w-killactive

```

2. Override the default keybind in `~/.config/hypr/conf/custom.conf` (so ML4W updates won't overwrite it):

```ini
unbind = $mainMod, Q
bind = $mainMod, Q, exec, ~/.local/bin/ml4w-killactive

```

> [!NOTE]
> - The Rust code is currently a bit (very) messy and very much a WIP - expect rough edges and a few (many) `.unwrap()`s here and there.
