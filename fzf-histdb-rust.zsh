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
    echo "You have to do it yourself"
  else
    curl -s -JLO https://github.com/m42e/zsh-histdb-skim/releases/download/$(histdb-skim-get-latest-version)/zsh-histdb-skim-$(histdb-skim-get-os)
  fi
}


histdb-fzf-widget() {
  origquery=${BUFFER}
  output=$( \
    HISTDB_HOST=$HISTDB_HOST \
    HISTDB_SESSION=$HISTDB_SESSION \
    HISTDB_FILE=$HISTDB_FILE \
    /work/zsh-histdb-rust/target/debug/zsh-histdb-rust "$origquery"\
  )

  if [ $? -eq 0 ]; then
    BUFFER=$output
  else
    BUFFER=$origquery
  fi

  CURSOR=$#BUFFER
  zle redisplay
}

zle     -N   histdb-fzf-widget
bindkey '^R' histdb-fzf-widget
