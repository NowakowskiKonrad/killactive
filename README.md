# ml4w-killactive

Instant `SUPER+Q` for ML4W / Hyprland.

`ml4w-killactive` closes Quickshell panels or the active window by talking directly to Hyprland and Quickshell over UNIX sockets. It avoids spawning `qs`, `hyprctl`, and `jq` on every keypress.

The original ML4W shell script has noticeable latency because each panel check spawns a Quickshell CLI process. This utility sends the IPC messages directly.

## What it does

Given rules like:

```bash
ml4w-killactive \
  -l sidebar:close \
  -l wallpaper:close \
  -l calendar:close \
  -w 'ML4W Welcome:welcome:toggle' \
  -w 'ML4W Dotfiles Settings:settings:toggle'
```

it will:

1. Check Hyprland layer surfaces for Quickshell processes.
2. Send the configured panel methods to those Quickshell processes.
3. If no panel path applies, inspect the active window.
4. Either send a configured window-specific IPC method or close the active window through Hyprland.

With no rules, it behaves like a plain active-window close command.

## Installation

Build and install to `~/.local/bin`:

```bash
make install
```

Install somewhere else:

```bash
make install PREFIX=$HOME/.config/hypr/scripts
```

Manual build:

```bash
cargo build --release
install -Dm755 target/release/ml4w-killactive ~/.local/bin/ml4w-killactive
```

## Hyprland keybind

Example override for `.config/hypr/conf/keybindings/default.lua`:

```lua
hl.unbind(mainMod .. " + Q")

hl.bind(
	mainMod .. " + Q",
	hl.dsp.exec_cmd(
		"~/.local/bin/ml4w-killactive -l sidebar:close "
			.. "-l wallpaper:close "
			.. "-l calendar:close "
			.. "-w 'ML4W Welcome:welcome:toggle' "
			.. "-w 'ML4W Dotfiles Settings:settings:toggle'"
	),
	{ description = "Kill active window" }
)
```

Prefer putting this in your own `custom.lua` so ML4W updates do not overwrite it.

If `~` is not expanded in your setup, use an absolute path.

## Usage

```bash
ml4w-killactive [-l target:method]... [-w title:target:method]...
```

Layer rule:

```bash
-l sidebar:close
```

Window rule:

```bash
-w 'ML4W Welcome:welcome:toggle'
```

Window titles may contain colons. Window rules are parsed from the right, so this works:

```bash
-w 'Some:Title:target:method'
```

## Assumptions and limitations

This is ML4W-first, not a universal Quickshell panel manager.

The default fast path makes one intentional assumption:

> If Hyprland reports a mapped layer surface whose namespace contains `quickshell`, treat that as “one of the configured panels is open” and send the configured layer methods.

That keeps the keybind extremely fast because it does not wait for a semantic `isOpen` response.

Consequences:

- It does **not** call `isOpen` by default.
- It assumes ML4W’s mapped Quickshell layer surfaces correspond to the panels you configure.
- If your setup has non-panel Quickshell layer surfaces, such as notifications or desktop widgets, the layer assumption may not hold.
- In that case, the tool may skip the active-window path even though no configured panel was actually open.
- IPC targets are expected to identify the intended panel. Sending a method to a Quickshell instance that does not own the target is treated as harmless.
- Window rules match exact Hyprland window titles.
- The Hyprland dispatch syntax assumes Hyprland 0.55+ Lua dispatchers.
- The Quickshell IPC wire format is unofficial and was recovered from socket captures. It may change between Quickshell versions.

If you need exact panel-state semantics for arbitrary Quickshell setups, you want a slower probe mode that calls `isOpen` and reads the reply. That is not the default path here.

## Benchmark

Run:

```bash
make bench
```

Requires:

- `hyperfine`
- `jq`
- executable scripts in `scripts/`

Reference run on my machine, empty workspace, no panel open:

| Implementation | Mean |
|---|---:|
| Bash original | 111.0 ms |
| Bash parallel proof-of-concept | 42.2 ms |
| Rust binary | 866 µs |

This benchmark measures process-spawn and IPC discovery overhead. It is not a full interactive panel-close benchmark. Re-run it after changes and update the numbers for your machine.

The important win is not “Rust is faster than Bash.” The win is avoiding repeated Quickshell CLI process spawns.

## Protocol notes

The Quickshell call frame used here is:

```text
u8 0x03
QString target
QString method
u32 argc
```

`QString` is encoded as:

```text
u32 BE byte length
UTF-16BE payload
```

Observed reply frames:

```text
close:
05 01 00 00 00 00

isOpen true:
05 00 00 00 00 08 00 74 00 72 00 75 00 65
```

The byte after `0x05` is not a simple success flag. The tests lock the request encoding; the reply format is documented from observed captures.

## Development

```bash
make test
make fmt
make lint
```

The tests cover:

- Quickshell request encoding
- UTF-16BE QString length handling
- Hyprland layer JSON filtering
- active-window JSON deserialization

## Disclaimer

`scripts/killactive_fast.sh` was written entirely by AI as a quick proof-of-concept to test parallel Bash overhead before rewriting the utility in Rust.