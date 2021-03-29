# Resources used by LODA Lab

When the "LODA Lab" executable is running, it makes use of the files in this dir.


### The `dont_mine.csv` file

The file `dont_mine.csv` contains the OEIS sequences that is not be mined.
This is used when running the `loda-lab update` command.

What goes into the `dont_mine.csv` file.

- Duplicate OEIS sequence.
- Existing programs.

The `dont_mine.csv` is extracted from the LODA `denylist.txt`.

To rebuild this file follow these steps:

```
PROMPT> pwd
/Users/JOHNDOE/git/loda-lab/script
PROMPT> rake data/dont_mine.csv
PROMPT> cd ..
PROMPT> cp script/data/dont_mine.csv resources/dont_mine.csv
```

