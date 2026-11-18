#!/bin/bash

OPEN_PANEL=$(
    for panel in sidebar wallpaper calendar; do
        ( [ "$(qs ipc call "$panel" isOpen 2>/dev/null)" = "true" ] && echo "$panel" ) &
    done
    wait
)

if [ -n "$OPEN_PANEL" ]; then
    qs ipc call "${OPEN_PANEL%%$'\n'*}" close >/dev/null 2>&1 &
    exit 0
fi

ACTIVE_JSON=$(hyprctl activewindow -j)
if [[ $ACTIVE_JSON =~ \"title\":[[:space:]]*\"(([^\"\\]|\\.)*)\" ]]; then
    ACTIVE_TITLE="${BASH_REMATCH[1]}"
fi

if [ -z "$ACTIVE_TITLE" ] || [ "$ACTIVE_TITLE" = "null" ]; then
    exit 0
fi

case "$ACTIVE_TITLE" in
    "ML4W Welcome")
        qs ipc call welcome toggle >/dev/null 2>&1 &
        ;;
    "ML4W Dotfiles Settings")
        qs -p "$HOME/.local/share/ml4w-dotfiles-settings/quickshell" ipc call settings toggle >/dev/null 2>&1 &
        ;;
    *)
        hyprctl dispatch 'hl.dsp.window.close()' >/dev/null 2>&1 &
        ;;
esac