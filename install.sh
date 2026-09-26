#!/usr/bin/env bash
# Build and install the binaries, helper scripts, and systemd user units.
# Re-run after any change; running services are restarted to pick it up.
set -euo pipefail
cd "$(dirname "$0")"

bin_dir="$HOME/.local/bin"
unit_dir="$HOME/.config/systemd/user"
units=(clipboard-tts.service tts-loop-watch.service)

cargo build --release
install -Dm755 -t "$bin_dir" target/release/clipboard-tts target/release/tts-loop-watch scripts/*
install -Dm644 -t "$unit_dir" "${units[@]/#/systemd/}"

systemctl --user daemon-reload
for unit in "${units[@]}"; do
  systemctl --user try-restart "$unit"
done
