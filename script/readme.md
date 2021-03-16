# Automation

These scripts for automation uses the [Ruby programming_language](https://www.ruby-lang.org/en/).


# Usage - Basics

### Show help

```
PROMPT> rake
rake data/bigram.csv                # generate a bigram
rake data/caller_callee_list.csv    # obtain all the dependencies between programs, comma separated list
rake data/caller_callee_pairs.csv   # obtain all the dependencies between programs, for use as input to PageRank algorithm
rake data/compare_loda_vs_lab.csv   # compare terms between "LODA official" and "LODA Lab"
rake data/most_called_programs.csv  # determine the most called programs
rake data/pagerank.csv              # run the PageRank algorithm and ranking the most influential programs
rake data/program_ids.csv           # obtain all the program ids
rake data/terms_lab.csv             # compute terms with "LODA Lab"
rake data/terms_loda.csv            # compute terms with "LODA Official"
PROMPT>
```

### Generate a CSV file with the most used programs

```
PROMPT> rake data/most_called_programs.csv
... snip, this task takes 3 minutes ...
PROMPT>
```

On successful completion there should be the file: `data/most_called_programs.csv`.


# Usage - Advanced

### Generate a CSV file with PageRank

This depends on the [PageRank tool](https://github.com/louridas/pagerank) being installed in the path.
See the `task_pagerank.rb` for instructions how to install the pagerank tool.

```
PROMPT> rake data/pagerank.csv
... snip, this task takes 3 minutes ...
PROMPT>
```

On successful completion there should be the file: `data/pagerank.csv`.

