# killactive

Instant `SUPER+Q` replacement for ML4W killactive.sh script, made out of pure hatred for slow bash scripts. Closes Quickshell overlay panels or the active window by talking directly to Hyprland and Quickshell over UNIX sockets—eliminating excessive process spawns on every keypress.

Measured on a laptop with AMD Ryzen 7 7735HS, plugged in. On battery the script was even slower (if I was in a hurry clicking between windows it closed wrong things... 😴😴😴).

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `Bash (original)` | 115.0 ± 1.4 | 112.5 | 119.3 | 132.89 ± 17.83 |
| `Bash (fast)` | 43.1 ± 0.5 | 42.4 | 44.1 | 49.74 ± 6.67 |
| `Rust (binary)` | 0.9 ± 0.1 | 0.7 | 1.1 | 1.00 |
| `Rust (faster binary, -f)` | 0.9 ± 0.1 | 0.7 | 1.1 | 0.99 ± 0.19 |

*Measured via `make bench` (`hyperfine`) on an empty workspace; captures process-spawn and IPC discovery overhead.*

*Note: `scripts/killactive_fast.sh` was an AI-generated parallel-Bash prototype used to benchmark shell overhead prior to the Rust rewrite.*

## How It Works

1. **Layer check:** Queries Hyprland `j/layers` for mapped surfaces whose namespace contains `quickshell`.
   - **Default:** Probes `isOpen` on each configured `-l` target first, only closing and exiting if one actually replies `"true"` (so persistent Quickshell bars or widgets won't eat `SUPER+Q`).
   - ***Faster* mode (`-f` / `--faster`):** Skips `isOpen` and immediately sends all configured `-l target:method` calls to those Quickshell PID sockets and exits.
2. **Active window fallback:** Queries `j/activewindow`. If the title matches a `-w` rule, sends that IPC call to the window's Quickshell PID; otherwise dispatches `hl.dsp.window.close()`. With no flags, it acts as a plain active-window closer.

## Installation

```bash
make install                                    # installs to ~/.local/bin
make install PREFIX=$HOME/.config/hypr/scripts  # custom destination
```

Or build manually with `cargo build --release` (binary at `target/release/killactive`).

OR my fellow arch users can just use PKGBUILD, you know how ;)

## Hyprland Keybind

Add to `~/.config/hypr/conf/keybindings/custom.lua` so ML4W updates do not overwrite it (ensure ~/.local/bin is in your PATH if using make install):

```lua
hl.unbind(mainMod .. " + Q")

hl.bind(
	mainMod .. " + Q",
	hl.dsp.exec_cmd(
		"killactive -l sidebar:close "
			.. "-l wallpaper:close "
			.. "-l calendar:close "
			.. "-w 'ML4W Welcome:welcome:toggle' "
			.. "-w 'ML4W Dotfiles Settings:settings:toggle'"
	),
	{ description = "Kill active window" }
)
```

## CLI Usage

```text
killactive [-f] [-h] [-l target:method] [-w title:target:method]
```

* `-f`, `--faster` — Skip probing `isOpen` and immediately send `-l` close calls whenever any `quickshell` layer surface is mapped (saves whole ~10 µs, it's about speed baby).
* `-l`, `--layer <target:method>` — IPC call sent to active Quickshell layer PIDs (e.g. `sidebar:close`).
* `-w`, `--window <title:target:method>` — IPC call sent when the active window's exact title matches (e.g. `'ML4W Welcome:welcome:toggle'`). Parsed from the right, so titles containing colons (`Some:Title:target:method`) work.

## Assumptions & Protocol Notes

- **Probes `isOpen` by default:** Safe to use alongside persistent Quickshell layer widgets (bars, notifications, desktop widgets). If you only use Quickshell for transient popups and want to skip the `isOpen` round-trip, pass `-f` / `--faster`.
- **Hyprland 0.55+:** Window closing uses Lua dispatcher syntax (`hl.dsp.window.close()`).
- **Unofficial Quickshell wire format:** Reverse-engineered from socket captures and subject to change across Quickshell versions:
  - **Request:** `u8 0x03` (`StringCallCommand`) + `QString` target + `QString` method + `u32` BE `argc` (`0`), where `QString` is `u32` BE byte length + UTF-16BE payload.
  - **Observed replies:** `05 01 00 00 00 00` (`close`) and `05 00 00 00 00 08 00 74 00 72 00 75 00 65` (`isOpen` → `"true"`).


## Development

```bash
make test   # run unit tests (wire encoding, UTF-16BE lengths, JSON parsing)
make fmt    # format with rustfmt
make lint   # run clippy (-D warnings)
make bench  # run hyperfine comparison (requires hyperfine and jq)
```