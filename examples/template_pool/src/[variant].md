# Template Pool

Template Pool allowed you to generate more files base on a specific template instead of just replacing its content.
And unlike Keys Template, you will use the `template` section of the config file to specify the templating expression.

## Access Limitation

You can only access the keyword from the template section if the path leading to this file contains an expression block with the same value as the keyword you want.
In this case, this file can only access the [variant] keyword but not the [color] keyword because the path leading to this file is `src/[variant].txt` which contain no reference to [color].