XDG_BIN_PATH=${XDG_DATA_HOME:-$HOME/.local/share}/zsh-histdb-skim/
BIN_DIR=${HISTDB_SKIM_PATH:-${XDG_BIN_PATH}}
BIN_PATH=${BIN_DIR}/zsh-histdb-skim

HISTB_SKIM_VERSION="v0.9.4"

histdb-skim-get-os(){
  UNAME_STR=`uname -a`
  if [[ ( $UNAME_STR =~ '.*Darwin.*' ) && ( $UNAME_STR =~ '.*x86_64.*') ]]; then
    echo -n "darwin-x64"
  fi
  if [[ ( $UNAME_STR =~ '.*Darwin.*' ) && ( $UNAME_STR =~ '.*arm64.*') ]]; then
    echo -n "darwin-arm"
  fi
  if [[ ( $UNAME_STR =~ '.*Linux.*' ) && ( $UNAME_STR =~ '.*x86_64.*') ]]; then
    echo -n "linux-x64"
  fi
}

histdb-skim-get-latest-version(){
  curl -s https://github.com/m42e/zsh-histdb-skim/releases/latest | grep  --color=never -o 'v[0-9]*\.[0-9]*\.[0-9]*'
}

histdb-skim-download(){
  if [[ -z $(histdb-skim-get-os) ]]; then
    echo "Could not find prebuild executable"
    echo "Sorry, you have to do it yourself"
  else
    echo "Downloading binary"
    mkdir -p ${BIN_DIR}
    curl -s -JL https://github.com/m42e/zsh-histdb-skim/releases/download/${HISTB_SKIM_VERSION}/zsh-histdb-skim-$(histdb-skim-get-os) -o ${BIN_PATH}
    chmod +x ${BIN_PATH}
  fi
}

histdb-skim-ensure () {
  if [[ ! -f ${BIN_PATH} || $(${BIN_PATH} --version) != ${HISTB_SKIM_VERSION} ]]; then
    if command -v cargo &> /dev/null; then
      echo "cargo is available, starting Rust release build"
      SCRIPT_DIR=$(cd "$(dirname "$0:A")" && pwd)
      (cd ${SCRIPT_DIR};\
      cargo build --release --manifest-path "${SCRIPT_DIR}/Cargo.toml" && \
      cp "${SCRIPT_DIR}/target/release/zsh-histdb-skim" ${BIN_PATH})
    else
      histdb-skim-download
    fi
  fi
}

histdb-skim-widget() {
  origquery=${BUFFER}
  output=$( \
    HISTDB_HOST=${HISTDB_HOST:-"'$(sql_escape ${HOST})'"} \
    HISTDB_SESSION=$HISTDB_SESSION \
    HISTDB_FILE=$HISTDB_FILE \
    ${BIN_PATH} "$origquery"\
  )

  if [ $? -eq 0 ]; then
    BUFFER=$output
  else
    BUFFER=$origquery
  fi

  CURSOR=$#BUFFER
  zle redisplay
}

histdb-skim-ensure

zle     -N   histdb-skim-widget
bindkey '^R' histdb-skim-widget
