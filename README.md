# Megumax

Megumax is a simple templating CLI originally made for developing minecraft datapack but the program should work anywhere else as well.

# Installation & Usage

1. Install [rustup](https://www.rust-lang.org/tools/install)
2. Install megumax using this command `cargo install megumax` (make sure you've restart your terminal first)
3. Create and configure your `megu.toml` file.
4. Run `megu` command inside the directory `megu.toml` file is in.

# Basic

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

For more information, check out `examples/` directory.

# Why?

When working with [custom model data](https://minecraft.gamepedia.com/Player.dat_format#General_Tags) you need to specify an integer value for each model in your resourcepack. This can become unmaintainable when working with models up to 100+. Megumax would help this by allowing you to specify a string value for each model instead and then compile it back into integers when needed.

There's probably already existing tool for this but why waste time googling for 3 seconds when you can spend your afternoon developing this instead? /s
