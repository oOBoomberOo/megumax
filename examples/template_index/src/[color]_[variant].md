Template Pool exposes a special key template called [nth] that will get replaced with the index of the generated file.

Due to the generation order, the [nth] template will be generated after the program has determined all the possible path variants so you can't use [nth] in the file name.

> Note that the program makes no guarantee of the order the file will generate in.