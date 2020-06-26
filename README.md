# Megumax

Megumax is a simple CLI tool for simple search and replace across the entire project and quickly generating multiple similar files from a template format.

The original intention for this program is for developing minecraft datapack but the program should work anywhere else as well.

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

This program will then:
1. Recursively find all of `replacer-placeholder`. (Note: `replacer-placeholder` is a term for a keyword surrounded by two braces. In the example above it will be `{{foo}}` and `{{bar}}`)
2. Replace all of `replacer-placeholder` instances with the value specified in the config file.
3. Output everything into `output` directory specified in the config file.

> Megumax will also respect .gitignore file and hidden files as well.

The advantage of this tool is that it doesn't replace the source code but rather, output it into different directory.

# TOML Format

```toml
[build]
output = "..." # Specify the output directory, Required.
src = "..." # Specify the source directory, Optional, will use current directory if not specified.

[keys] # List of master replacer keys
key_name_1 = "some_value_1"
key_name_2 = "some_value_2"
key_name_3 = "some_value_3"

[[template]]
input = "..." # Glob Pattern for input files, Required.
output = "..." # Relative Path for output files, Required, `values` field will be apply to this path as well.
values = [ # List of replacer, Required.
	{ foo = '001' },
	{ foo = '002' },
]
```

# Why?

When working with [custom model data](https://minecraft.gamepedia.com/Player.dat_format#General_Tags) you need to specify an integer value for each model in your resourcepack. This can become unmaintainable when working with models up to 100+. Megumax would help this by allowing you to specify a string value for each model instead and then compile it back into integers when needed.

There's probably already existing tool for this but why waste time googling for 3 seconds when you can spend your afternoon developing this instead? /s

# Template (Advanced Usage)

This section will introduce you to a "Template" concept of Megumax.

Template is useful for generating multiple similar looking files. This can heavily reduce repetitive work for you.

To use Template, you must first define a template array inside the config file.
```toml
# ...

[[template]]
input = "**/foo.template"
output = "../{{key}}.json"
values = [
	{ key = 'foo' },
	{ key = 'bar' },
	{ key = 'baz' },
]
```

1. `input`
This is a "glob pattern" which you can use to specify which file will be count as template file. I recommend that you use `.template` extension to indicate a template file as well to prevent any accidental file selection.

2. `output`
This is an output path which specify where to output *each* instance of this template. You can specify the `replacer-placeholder` in here as well.  
And since this is a relative path, accessing parent or even a whole another directory is allowed (even non-existing directory).

3. `values`
This is a list of "replacer" which will create multiple instance of template. Each generated template will apply the replacer specified in the template list __and__ the master replacer. (The one specified under `[keys]` section)

## Pre-defined keys

- `{{filename}}`: the name of the current file.
- `{{filestem}}`: the name of the current file __without__ file extension.
- `{{extension}}`: the extension of the current file.
