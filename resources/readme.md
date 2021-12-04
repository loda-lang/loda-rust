# Resources used by LODA Rust

When the "LODA Lab" executable is running, it makes use of the files in this dir.


### The `program_creation_dates.csv` file

Date when a LODA program got added to the git repository.

This is useful when randomly picking a fairly recent `program_id`.

This task is VERY time consuming. It took 15 hours to extract creation date from 28k files.

To rebuild this file follow these steps:

```
PROMPT> pwd
/Users/JOHNDOE/git/loda-rust/script
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
/Users/JOHNDOE/git/loda-rust/script
PROMPT> rake data/program_popularity.csv
PROMPT> cd ..
PROMPT> cp script/data/program_popularity.csv resources/program_popularity.csv
```


