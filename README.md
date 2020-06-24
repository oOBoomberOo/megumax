# Megumax

Megumax is a simple CLI tool for quickly generating a template project templatew ith simple search and replace functionality.

The original intention for this program is for developing minecraft datapack but the program should work anywhere else as well.

## Basic

Megumax work via a config file called `megu.toml` which look something like this:

```toml
[build]
output = "path/to/output/directory"
src = "path/to/src/directory"

[keys]
foo = "Kore wa requiem da"
bar = "42"
```

And then you can run the program inside the directory this config file is in with `megu` command. (No need for any argument)

This program will then recursively find all text files and replace all of `{{key}}` instances with the value specified in the config file. (Note: `key` is a placeholder value, in the example above it will be `{{foo}}` and `{{bar}}`) **and then** it will output all of that into the `output` directory specified in the config file.  
Megumax will also respect .gitignore file and hidden files as well.

The advantage of this tool is that it doesn't replace the source code but rather, output it into different directory.

## TOML Format

```toml
[build]
output = "..." # Specify the output directory, Required.
src = "..." # Specify the source directory, Optional, will use current directory if not specified.

[keys] # List of template keys
key_name_1 = "some_value_1"
key_name_2 = "some_value_2"
key_name_3 = "some_value_3"
```

## Why?

When working with [custom model data](https://minecraft.gamepedia.com/Player.dat_format#General_Tags) you need to specify an integer value for each model in your resourcepack. This can become unmaintainable when working with models up to 100+. Megumax would help this by allowing you to specify a string value for each model instead and then compile it back into integers when needed.

There's probably already existing tool for this but why waste time googling for 3 seconds when you can spend your afternoon developing this instead? /s

## Installation & Usage

1. Install [rustup](https://www.rust-lang.org/tools/install)
2. Install megumax using this command `cargo install megumax` (make sure you've restart your terminal first)
3. Create and configure your `megu.toml` file.
4. Run `megu` command inside the directory `megu.toml` file is in.
