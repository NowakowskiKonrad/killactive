# ml4w-killactive

Quick little program to make closing apps and panels in ML4W dotfiles instant.

The original shell script has a noticeable lag (~115 ms) from spawning multiple `qs`, `hyprctl`, and `jq` processes. This utility talks directly to Quickshell (`QDataStream`) and Hyprland over UNIX sockets via IPC.

## Benchmark

```text
--------------------------------------------------------------------------
Bash (original)  | avg: 115.839 ms | min: 113.651 ms | max: 121.230 ms
Bash (fast)      | avg:  43.890 ms | min:  42.920 ms | max:  45.104 ms
Rust (binary)    | avg:   2.966 ms | min:   2.519 ms | max:   3.730 ms
--------------------------------------------------------------------------
Bash (fast) vs original :   2.6x faster (saves  71.95 ms)
Rust vs Bash (fast)     :  14.8x faster (saves  40.92 ms)
Rust vs Bash (original) :  39.0x faster (saves 112.87 ms)

```

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
> The code is currently a bit messy and very much a WIP — expect rough edges and a few `.unwrap()`s here and there.