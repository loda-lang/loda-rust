# Using "loda-rust"

Question: Can loda-rust update oeis index, and programs repository by itself?

Answer: Currently I’m doing it manually. Over time I hope to automate it.

My daily routine is something like this:

1. Stop `loda-rust mine` if it's running.
2. Fetch the latest `loda-programs` repo.
3. Run `loda-rust analytics` to update histograms/bloomfilters.
4. Run `loda-rust mine`, this outputs candiate programs into the dir: `~/.loda-rust/mine-event`.

When there are around 400 candidate programs inside the `mine-event` dir:
```
PROMPT> cd loda-rust/script
PROMPT> rake process_mined_programs
snip .. takes around 2 hours for 400 candidate programs .. snip
PROMPT>
```

For 400 candidate programs, it takes around 2 hours to analyze the candate programs. 
It determines: Is it the correct terms, is the program faster than the existing program.
It places the programs inside the `loda-programs` repo, but it doesn't do any commit.

1. I manually inspect all diffs in the `loda-programs` repo. 
2. Is it one-line that has been inserted with a `mod $0,12345`, then I discard it.
3. Has magic constants been introduced, then I discard it.
4. Then commit it with the commit message: "Updated programs".

Then run the script
```
PROMPT> cd loda-rust/script
PROMPT> ruby upload_program_files_to_server_from_commit.rb
snip .. takes about 20 seconds for 200 programs .. snip
PROMPT>
```

Now the programs have been uploaded to the loda-lang.org server.
