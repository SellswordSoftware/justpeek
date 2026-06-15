#!/bin/bash

set -euo pipefail

CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/justpeek/references"

mkdir -p "$CONFIG_DIR"
cp -R examples/* "$CONFIG_DIR/"

echo "References directory created at: $CONFIG_DIR"
echo "Example files copied."
