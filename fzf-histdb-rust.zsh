histdb-fzf-widget() {
  local selected num mode exitkey typ cmd_opts cmd_opts_extra
  origquery=${BUFFER}
  output=$(HISTDB_HOST=$HISTDB_HOST HISTDB_SESSION=$HISTDB_SESSION /work/zsh-histdb-rust/target/debug/zsh-histdb-rust "$origquery")

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
