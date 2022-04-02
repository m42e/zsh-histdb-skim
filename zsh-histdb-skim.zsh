THIS_PATH=${0:a:h}
XDG_BIN_PATH=${XDG_DATA_HOME:-$HOME/.local/share}/zsh-histdb-skim/
BIN_DIR=${HISTDB_SKIM_PATH:-${XDG_BIN_PATH}}
BIN_PATH=${BIN_DIR}/zsh-histdb-skim

histdb-skim-get-os(){
  UNAME_STR=`uname -a`
  if [[ ( $UNAME_STR =~ '.*Darwin.*' ) && ( $UNAME_STR =~ '.*x86_64.*') ]]; then
    echo -n "darwin-x64"
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
    curl -s -JL https://github.com/m42e/zsh-histdb-skim/releases/download/$(histdb-skim-get-latest-version)/zsh-histdb-skim-$(histdb-skim-get-os) -o ${BIN_PATH}
    chmod +x ${BIN_PATH}
  fi
}


histdb-skim-ensure () {
  if [[ ! -f ${BIN_PATH} ]]; then
    histdb-skim-download
  fi
  if [[ ${BIN_PATH} != "v0.7.0" ]]; then
    histdb-skim-download
  fi
}

histdb-skim-widget() {
  histdb-skim-ensure
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

zle     -N   histdb-skim-widget
bindkey '^R' histdb-skim-widget
