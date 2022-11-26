#!/usr/bin/env zsh

histdb-skim-widget() {
  origquery=${BUFFER}
  output=$( \
    HISTDB_HOST=${HISTDB_HOST:-"'$(sql_escape ${HOST})'"} \
    HISTDB_SESSION=$HISTDB_SESSION \
    HISTDB_FILE=$HISTDB_FILE \
    zsh-histdb-skim "$origquery"\
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
