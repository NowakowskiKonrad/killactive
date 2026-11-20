# ml4w-killactive

Quick little program to make closing apps and panels in ML4W dotfiles instant.

The original shell script has a noticeable lag (~115 ms) from spawning multiple `qs`, `hyprctl`, and `jq` processes. This utility talks directly to Quickshell (`QDataStream`) and Hyprland over UNIX sockets via IPC.

## Benchmark

```text
Benchmark 1: Bash (original)
  Time (mean ± σ):     111.0 ms ±   1.8 ms    [User: 78.3 ms, System: 33.8 ms]
  Range (min … max):   108.3 ms … 117.7 ms    50 runs
 
Benchmark 2: Bash (fast)
  Time (mean ± σ):      42.2 ms ±   0.5 ms    [User: 75.9 ms, System: 34.2 ms]
  Range (min … max):    41.4 ms …  44.3 ms    50 runs
 
Benchmark 3: Rust (binary)
  Time (mean ± σ):     866.0 µs ± 108.0 µs    [User: 280.4 µs, System: 467.3 µs]
  Range (min … max):   647.4 µs … 1038.8 µs    50 runs
 
Summary
  Rust (binary) ran
   48.72 ± 6.11 times faster than Bash (fast)
  128.20 ± 16.12 times faster than Bash (original)

```
*Disclaimer: `scripts/killactive_fast.sh` was written entirely by AI as a quick proof-of-concept to test parallel Bash overhead before rewriting the utility in Rust.*

## Installation & Usage

1. Build and copy the binary:

```bash
cargo build --release
cp target/release/ml4w-killactive ~/.local/bin/ml4w-killactive

```

2. Override the default keybind in `.config/hypr/conf/keybindings/default.lua` (The best thing to do is create own `custom.lua` file and use it instead, because the default one can get overridden):

```lua
hl.unbind(mainMod .. " + Q")
hl.bind(
	mainMod .. " + Q",
	hl.dsp.exec_cmd(
		"~/.config/hypr/scripts/ml4w-killactive -l sidebar:close "
			.. "-l wallpaper:close "
			.. "-l calendar:close "
			.. "-w 'ML4W Welcome:welcome:toggle' "
			.. "-w 'ML4W Dotfiles Settings:settings:toggle'"
	),
	{ description = "Kill active window" }
)


```
