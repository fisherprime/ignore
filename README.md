# ignore

[![pipeline status](https://gitlab.com/fisherprime/ignore/badges/master/pipeline.svg)](https://gitlab.com/fisherprime/ignore/-/commits/master)

`ignore` is a tool used to generate gitignore files from user defined gitignore template sources,

# Install

- Clone the respoitory with `git clone --depth 1 https://gitlab.com/fisherprime/ignore`
- Execute`cargo install --path .` to install the binary at `$CARGO_HOME`.

# Usage

On initial execution, a config file will be generated & stored at
`$XDG_CONFIG_HOME/ignore/config.toml` or your OS' specific config directory. The template sources &
other options can be configured in this file.

`ignore -h` will display the binary's usage instructions.

# Sample sources

**Currently works with git repositories**

Gitignore template source examples:

    - [https://github.com/toptal/gitignore](Gitignore.io templates)
    - [https://github.com/github/gitignore](Github templates)
