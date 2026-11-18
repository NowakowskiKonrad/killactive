#!/usr/bin/env bash
set -euo pipefail

BASH_SCRIPT="./scripts/killactive.sh"
BASH_SCRIPT_FAST="./scripts/killactive_fast.sh"
RUST_BIN="./target/release/ml4w-killactive"
WARMUP=5
RUNS=50

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

bench() {
    local name="$1"
    local cmd="$2"
    local total=0
    local min=999999999999
    local max=0

    for _ in $(seq 1 "$WARMUP"); do
        "$cmd" >/dev/null 2>&1 || true
    done

    for _ in $(seq 1 "$RUNS"); do
        local start end diff
        start=$(date +%s%N)
        "$cmd" >/dev/null 2>&1 || true
        end=$(date +%s%N)

        diff=$((end - start))
        total=$((total + diff))
        if ((diff < min)); then min=$diff; fi
        if ((diff > max)); then max=$diff; fi
    done

    awk -v name="$name" -v total="$total" -v min="$min" -v max="$max" -v runs="$RUNS" '
    BEGIN {
        printf "%-16s | avg: %7.3f ms | min: %7.3f ms | max: %7.3f ms\n",
            name, (total / runs) / 1e6, min / 1e6, max / 1e6
    }' >&2

    echo $((total / RUNS))
}

echo "--------------------------------------------------------------------------"

ns_bash=$(bench "Bash (original)" "$BASH_SCRIPT")
ns_bash_fast=$(bench "Bash (fast)" "$BASH_SCRIPT_FAST")
ns_rust=$(bench "Rust (binary)" "$RUST_BIN")

echo "--------------------------------------------------------------------------"
awk -v b="$ns_bash" -v bf="$ns_bash_fast" -v r="$ns_rust" 'BEGIN {
    printf "Bash (fast) vs original : %5.1fx faster (saves %6.2f ms)\n", b / bf, (b - bf) / 1e6
    printf "Rust vs Bash (fast)     : %5.1fx faster (saves %6.2f ms)\n", bf / r, (bf - r) / 1e6
    printf "Rust vs Bash (original) : %5.1fx faster (saves %6.2f ms)\n", b / r, (b - r) / 1e6
}'