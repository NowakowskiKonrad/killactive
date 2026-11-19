# ml4w-killactive

Quick little program to make closing apps and panels in ML4W dotfiles instant.

The original shell script has a noticeable lag (~115 ms) from spawning multiple `qs`, `hyprctl`, and `jq` processes. This utility talks directly to Quickshell (`QDataStream`) and Hyprland over UNIX sockets via IPC.

## Benchmark

```text
Benchmark 1: Bash (original)
  Time (mean ± σ):     116.5 ms ±   1.7 ms    [User: 81.1 ms, System: 36.4 ms]
  Range (min … max):   114.3 ms … 125.8 ms    50 runs
 
Benchmark 2: Bash (fast)
  Time (mean ± σ):      43.4 ms ±   0.6 ms    [User: 78.3 ms, System: 35.1 ms]
  Range (min … max):    42.3 ms …  45.2 ms    50 runs
 
Benchmark 3: Rust (binary)
  Time (mean ± σ):     894.8 µs ± 117.2 µs    [User: 273.6 µs, System: 496.0 µs]
  Range (min … max):   670.5 µs … 1122.4 µs    50 runs
 
Summary
  Rust (binary) ran
   48.47 ± 6.39 times faster than Bash (fast)
  130.14 ± 17.15 times faster than Bash (original)

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