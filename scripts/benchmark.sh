#!/usr/bin/env bash
set -euo pipefail

BASH_SCRIPT="./scripts/killactive.sh"
BASH_SCRIPT_FAST="./scripts/killactive_fast.sh"
RUST_BIN="./target/release/killactive"
RUST_ARGS=" -l sidebar:close -l wallpaper:close -l calendar:close -w 'ML4W Welcome:welcome:toggle' -w 'ML4W Dotfiles Settings:settings:toggle'"

WARMUP=10
RUNS=50

if ! command -v hyperfine >/dev/null 2>&1; then
  echo "hyperfine is not installed. Install it with: sudo dnf install hyperfine"
  exit 1
fi

if [[ ! -x "$BASH_SCRIPT" || ! -x "$BASH_SCRIPT_FAST" || ! -x "$RUST_BIN" ]]; then
  echo "scripts are not executable, make sure that you ran:"
  echo "  chmod +x $BASH_SCRIPT $BASH_SCRIPT_FAST && cargo build --release"
  exit 1
fi

ORIG_WS=$(hyprctl activeworkspace -j | jq -r '.id')
cleanup() {
  hyprctl dispatch "hl.dsp.focus({ workspace = \"$ORIG_WS\" })" >/dev/null 2>&1 || true
}
trap cleanup EXIT

hyprctl dispatch 'hl.dsp.focus({ workspace = "99" })' >/dev/null
sleep 0.2

hyperfine \
  --shell=none \
  --warmup "$WARMUP" \
  --runs "$RUNS" \
  --export-markdown - \
  --command-name "Bash (original)" "$BASH_SCRIPT" \
  --command-name "Bash (fast)" "$BASH_SCRIPT_FAST" \
  --command-name "Rust (binary)" "$RUST_BIN $RUST_ARGS" \
  --command-name "Rust (faster binary)" "$RUST_BIN $RUST_ARGS -f"

