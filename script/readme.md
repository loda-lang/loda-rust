# Automation

These scripts for automation uses the [Ruby programming_language](https://www.ruby-lang.org/en/).

Run `bundle install` to install the dependencies listed in the `Gemfile`.


# Usage - Basics

### Show help

```
PROMPT> rake
rake cleanup_mismatch_filenames              # clean up the inconsistent filenames in the dir for mismatches
rake data/caller_callee_list.csv             # obtain all the dependencies between programs, comma separated list
rake data/compare_loda_cpp_vs_loda_rust.csv  # compare terms between "loda-cpp" and "loda-rust"
rake data/loda-rust                          # compiles the loda-rust executable
rake data/most_called_programs.csv           # determine the most called programs
rake data/program_creation_dates.csv         # extract creation date for all programs
rake data/program_ids.csv                    # obtain all the program ids
rake data/program_popularity.csv             # extract the most popular programs
rake data/terms_loda_cpp.csv                 # compute terms with "loda-cpp"
rake data/terms_loda_rust.csv                # compute terms with "loda-rust"
rake data/top100.md                          # create a markdown document with the 100 most popular LODA programs
PROMPT>
```

### Generate a CSV file with the most used programs

```
PROMPT> rake data/most_called_programs.csv
... snip, this task takes 3 minutes ...
PROMPT>
```

On successful completion there should be the file: `data/most_called_programs.csv`.

