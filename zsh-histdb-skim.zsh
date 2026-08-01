XDG_BIN_PATH=${XDG_DATA_HOME:-$HOME/.local/share}/zsh-histdb-skim/
BIN_DIR=${HISTDB_SKIM_PATH:-${XDG_BIN_PATH}}
BIN_PATH=${BIN_DIR}/zsh-histdb-skim

HISTB_SKIM_VERSION="v0.9.6"

histdb-skim-get-os(){
  local os_name os_arch
  os_name=$(uname -s)
  os_arch=$(uname -m)
  case "${os_name}:${os_arch}" in
    Darwin:x86_64) print -n -- "darwin-x64" ;;
    Darwin:arm64) print -n -- "darwin-arm" ;;
    Linux:x86_64) print -n -- "linux-x64" ;;
  esac
}

histdb-skim-get-latest-version(){
  curl -s https://github.com/m42e/zsh-histdb-skim/releases/latest | grep  --color=never -o 'v[0-9]*\.[0-9]*\.[0-9]*'
}

histdb-skim-download(){
  local os
  os=$(histdb-skim-get-os)
  if [[ -z "$os" ]]; then
    echo "Could not find prebuild executable"
    echo "Sorry, you have to do it yourself"
    return 1
  else
    echo "Downloading binary"
    mkdir -p "$BIN_DIR" || return 1
    if curl --fail --silent --show-error --location \
      "https://github.com/m42e/zsh-histdb-skim/releases/download/${HISTB_SKIM_VERSION}/zsh-histdb-skim-${os}" \
      --output "$BIN_PATH"; then
      chmod +x "$BIN_PATH"
    else
      rm -f "$BIN_PATH"
      return 1
    fi
  fi
}

histdb-skim-ensure () {
  if [[ ! -f "$BIN_PATH" || $("$BIN_PATH" --version 2>/dev/null) != "$HISTB_SKIM_VERSION" ]]; then
    if command -v cargo &> /dev/null; then
      echo "cargo is available, starting Rust release build"
      local source script_dir
      source="${(%):-%x}"
      script_dir=$(cd "$(dirname "$source")" && pwd) || return 1
      mkdir -p "$BIN_DIR" || return 1
      (cd "$script_dir" && \
      cargo build --release --manifest-path "$script_dir/Cargo.toml" && \
      cp "$script_dir/target/release/zsh-histdb-skim" "$BIN_PATH" && \
      chmod +x "$BIN_PATH")
    else
      histdb-skim-download
    fi
  fi
}

histdb-skim-widget() {
  local origquery output histdb_host
  origquery=$BUFFER
  histdb_host=${HISTDB_HOST:-}
  if [[ -z "$histdb_host" ]]; then
    histdb_host="'$(sql_escape "$HOST")'"
  fi

  if output=$( \
    HISTDB_HOST="$histdb_host" \
    HISTDB_SESSION="$HISTDB_SESSION" \
    HISTDB_FILE="$HISTDB_FILE" \
    "$BIN_PATH" "$origquery"\
  ); then
    BUFFER=$output
  else
    BUFFER=$origquery
  fi

  CURSOR=${#BUFFER}
  zle redisplay
}

histdb-skim-ensure

zle     -N   histdb-skim-widget
bindkey '^R' histdb-skim-widget
