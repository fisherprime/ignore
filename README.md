# ignore

[![pipeline status](https://gitlab.com/fisherprime/ignore/badges/master/pipeline.svg)](https://gitlab.com/fisherprime/ignore/-/commits/master)

`ignore` is a tool designed to generate `.gitignore` files based on user-defined gitignore templates.

# Install

- `cargo install --git https://gitlab.com/fisherprime/ignore`

# Usage

During the first execution, a configuration file will be generated and saved at either `$XDG_CONFIG_HOME/ignore/config.toml` (if available) or the default configuration directory for your operating system. The template sources and other options can be configured in this file.

`ignore -h` will display the binary's usage instructions.

# Sample sources

**Currently works with git repositories**

Gitignore template source examples:

    - [https://github.com/toptal/gitignore](Gitignore.io templates)
    - [https://github.com/github/gitignore](Github templates)
