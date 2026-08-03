#!/bin/sh
set -eu
repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
bin_dir=${OFFCODEX_INSTALL_DIR:-"$HOME/.local/bin"}
cd "$repo_root/codex-rs"
cargo build --release -p codex-cli
mkdir -p "$bin_dir"
install -m 0755 target/release/offcodex "$bin_dir/offcodex"
printf "Installed Offcodex at %s/offcodex\n" "$bin_dir"
printf "Run: %s/offcodex\n" "$bin_dir"
