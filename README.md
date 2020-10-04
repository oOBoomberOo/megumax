# Megumax

Megumax is a simple templating CLI originally made for developing minecraft datapack but the program should work anywhere else as well.

## Installation & Usage

### Using Cargo

1. Install [rustup](https://www.rust-lang.org/tools/install).
2. Install megumax using this command `cargo install megumax` (make sure you've restart your terminal first)
3. Create and configure your `megu.toml` file.
4. Run `megu` command inside the directory `megu.toml` file is in.

### Using pre-compiled file

1. Go to the [Release Page](https://github.com/oOBoomberOo/megumax/releases).
2. Download megumax base on your operating system.
3. Install the executable file, this process will be different for each OS.
    - Linux: Put the executable file inside `/usr/bin/` and set its execution permission.
	- Windows: Add the location of the executable file to the `$PATH` registry.
	- Mac: N/A

## Basic

Megumax work via a config file called `megu.toml` which look something like this:

```toml
[build]
output = "path/to/output/directory"
src = "path/to/src/directory"

[keys]
foo = "Kore wa requiem da"
bar = "42"

[template]
color = ["red", "orange", "yellow"]
```

And then you can run the program inside the directory this config file is in with `megu` command. (No need for any argument)

For more information, check out `examples/` directory.

## Why?

When working with [custom model data](https://minecraft.gamepedia.com/Player.dat_format#General_Tags) you need to specify an integer value for each model in your resourcepack. This can become unmaintainable when working with models up to 100+. Megumax would help this by allowing you to specify a string value for each model instead and then compile it back into integers when needed.

There's probably already existing tool for this but why waste time googling for 3 seconds when you can spend your afternoon developing this instead? /s

## Interface

<center>

![](https://i.imgur.com/6y47Wu5.png)

</center>