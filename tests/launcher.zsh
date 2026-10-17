#!/usr/bin/env zsh
# Offline regression checks for release selection and atomic downloads.
set -eu
repo_dir=${0:A:h:h}
test_dir=$(mktemp -d)
trap 'rm -rf "$test_dir"' EXIT
export HISTDB_SKIM_PATH="$test_dir/bin with spaces"
mkdir -p "$HISTDB_SKIM_PATH"
print -r -- '#!/bin/sh
printf "v0.9.6\n"' > "$HISTDB_SKIM_PATH/zsh-histdb-skim"
chmod +x "$HISTDB_SKIM_PATH/zsh-histdb-skim"
zle() { :; }
bindkey() { :; }
source "$repo_dir/zsh-histdb-skim.zsh"
uname() {
  case "$1" in
    -s) print Darwin ;;
    -m) print arm64 ;;
  esac
}
[[ $(histdb-skim-get-os) == darwin-aarch64 ]]
# Verify that the downloader's suffix names a published release asset.
asset="asset_name: zsh-histdb-skim-$(histdb-skim-get-os)"
[[ $(<"$repo_dir/.github/workflows/rust.yml") == *"$asset"* ]]

curl() {
  print -r -- "$download_content" > "${@[-1]}"
  return "$download_status"
}
original=$(<"$BIN_PATH")
download_content=partial
download_status=22
if histdb-skim-download; then
  print -u2 'Failed download unexpectedly succeeded'
  exit 1
fi
[[ $(<"$BIN_PATH") == "$original" ]]
[[ -x "$BIN_PATH" ]]
files=("$BIN_DIR"/*(N))
[[ ${#files} == 1 ]]

download_content=replacement
download_status=0
histdb-skim-download
[[ $(<"$BIN_PATH") == replacement ]]
[[ -x "$BIN_PATH" ]]
files=("$BIN_DIR"/*(N))
[[ ${#files} == 1 ]]
print 'Launcher checks passed'
