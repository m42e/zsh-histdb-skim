
```
#    #   ##   #####  #    # # #    #  ####  
#    #  #  #  #    # ##   # # ##   # #    # 
#    # #    # #    # # #  # # # #  # #      
# ## # ###### #####  #  # # # #  # # #  ### 
##  ## #    # #   #  #   ## # #   ## #    # 
#    # #    # #    # #    # # #    #  ####
```

This is currently work in progress. You have to compile it yourself. Works on my machine (contains hard coded paths). 


# zsh-histdb-skim

This is a reimplementation of https://github.com/m42e/zsh-histdb-fzf in rust and using skim as a library.

## Why

[zsh-histdb-fzf](https://github.com/m42e/zsh-histdb-fzf) works, but it is a bit quirky. It has for sure some flaws, regarding responsiveness and communication with processes.
[skim](https://github.com/lotabout/skim) offers a fzf like behavior and is available as library in rust.

This should result in better performance, responsiveness and a more stable behavior.

## Why rust?

skim is available in rust. I have never tried rust with a purpose before. I wanted to give it a try and learn something new.

## How it works

## Building

```
cargo build
```

# TODO
- Describe installation
- Add builds in github actions
- provide releases
- automate download of executable
- improve rust code

