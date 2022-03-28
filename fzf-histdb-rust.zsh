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
