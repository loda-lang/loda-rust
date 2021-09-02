# Automation

These scripts for automation uses the [Ruby programming_language](https://www.ruby-lang.org/en/).

Run `bundle install` to install the dependencies listed in the `Gemfile`.


# Usage - Basics

### Show help

```
PROMPT> rake
rake data/bigram.csv                         # generate a bigram
rake data/caller_callee_list.csv             # obtain all the dependencies between programs, comma separated list
rake data/caller_callee_pairs.csv            # obtain all the dependencies between programs, for use as input to PageRank algorithm
rake data/compare_loda_cpp_vs_loda_rust.csv  # compare terms between "loda-cpp" and "loda-rust"
rake data/denylist.csv                       # extract program ids from the LODA denylist file
rake data/dont_mine.csv                      # determine which program ids that shouldn't be attempted mined
rake data/loda-rust                          # compiles the loda-rust executable
rake data/mine_program_ids.csv               # identify the programs that can be used by the miner
rake data/most_called_programs.csv           # determine the most called programs
rake data/pagerank.csv                       # run the PageRank algorithm and ranking the most influential programs
rake data/program_creation_dates.csv         # extract creation date for all programs
rake data/program_ids.csv                    # obtain all the program ids
rake data/program_popularity.csv             # extract the most popular programs
rake data/skipgram.csv                       # generate a skipgram
rake data/terms_loda_cpp.csv                 # compute terms with "loda-cpp"
rake data/terms_loda_rust.csv                # compute terms with "loda-rust"
rake data/top100.md                          # create a markdown document with the 100 most popular LODA programs
rake data/trigram.csv                        # generate a trigram
PROMPT>
```

### Generate a CSV file with the most used programs

```
PROMPT> rake data/most_called_programs.csv
... snip, this task takes 3 minutes ...
PROMPT>
```

On successful completion there should be the file: `data/most_called_programs.csv`.


### Generate a CSV file with PageRank

This depends on the [PageRank tool](https://github.com/louridas/pagerank) being installed in the path.
See the `task_pagerank.rb` for instructions how to install the pagerank tool.

```
PROMPT> rake data/pagerank.csv
... snip, this task takes 3 minutes ...
PROMPT>
```

On successful completion there should be the file: `data/pagerank.csv`.

