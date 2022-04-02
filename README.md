# zsh-histdb-skim

This is a reimplementation of https://github.com/m42e/zsh-histdb-fzf in rust and using skim as a library.

## Why

[zsh-histdb-fzf](https://github.com/m42e/zsh-histdb-fzf) works, but it is a bit quirky. It has for sure some flaws, regarding responsiveness and communication with processes.
[skim](https://github.com/lotabout/skim) offers a fzf like behavior and is available as library in rust. It lacks some highlighting in the header, and has a bit different order/matching algorithm.

This should result in better performance, responsiveness and a more stable behavior.

## Why rust?

[skim](https://github.com/lotabout/skim) is available in rust. I have never tried rust with a purpose before. I wanted to give it a try and learn something new.

## How it works

Well, it accesses the [zsh histdb](https://github.com/larkery/zsh-histdb). It lets you search on different levels.

## What do you have to do?

Install the plugin, e.g. using [zplug](https://github.com/zplug/zplug).

```
  zplug 'm42e/zsh-histdb-skim', from:github, at:main
```

It downloads the binary (if available) automatically. You can do manually by calling `histdb-skim-download`. It will be saved in `${XDG_DATA_HOME}/zsh-histdb-skim`, alternatively `${HOME}/.local/share/zsh-histdb-skim`. You can specify the directory manually by setting `HISTDB_SKIM_PATH`.

The download will happen if the executable is not there or the version is outdated (starting from v0.7.0). These checks happen when sourcing the script.


The plugin calls `bindkey` but some other plugins may overwrite. In this case you would have to do it yourself:

```
bindkey '^R' histdb-skim-widget
```


## Additional information

By default the binary is downloaded


## Building

```
cargo build --release
mkdir -p bin
mv target/release/zsh-histdb-skim bin
```

# TODO
- improve rust code

# Apologies 😉

While I stole the idea from myself, this is my first rust project ever. So I would be glad for tips and improvement PRs.
