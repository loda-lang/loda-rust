# Resources used by LODA Lab

When the "LODA Lab" executable is running, it makes use of the files in this dir.


### The `dont_mine.csv` file

The file `dont_mine.csv` contains the OEIS sequences that is not to be mined.
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

### The `mine_program_ids.csv` file

These are the `program_ids` that are safe for the miner to use.

This list contains programs that can compute 10 terms.

This list does not contain:
- programs with cyclic dependencies.
- programs that fails to compute 10 terms.
 
So that the miner doesn't waste cpu on resolving cyclic dependencies.

To rebuild this file follow these steps:

```
PROMPT> pwd
/Users/JOHNDOE/git/loda-lab/script
PROMPT> rake data/mine_program_ids.csv
PROMPT> cd ..
PROMPT> cp script/data/mine_program_ids.csv resources/mine_program_ids.csv
```


### The `program_creation_dates.csv` file

Date when a LODA program got added to the git repository.

This is useful when randomly picking a fairly recent `program_id`.

This task is VERY time consuming. It took 15 hours to extract creation date from 28k files.

To rebuild this file follow these steps:

```
PROMPT> pwd
/Users/JOHNDOE/git/loda-lab/script
PROMPT> rake data/program_creation_dates.csv
PROMPT> cd ..
PROMPT> cp script/data/program_creation_dates.csv resources/program_creation_dates.csv
```


### The `program_popularity.csv` file

The most/least used LODA programs.

This is useful when randomly picking a fairly popular `program_id`.

To rebuild this file follow these steps:

```
PROMPT> pwd
/Users/JOHNDOE/git/loda-lab/script
PROMPT> rake data/program_popularity.csv
PROMPT> cd ..
PROMPT> cp script/data/program_popularity.csv resources/program_popularity.csv
```


