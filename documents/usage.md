# Using `loda-rust` for mining

Question: Can `loda-rust mine` run by itself?

Answer: No, currently I’m doing it manually. Over time I want to automate it.

My daily routine:

1. I observe the official `loda-programs` repo. There are usually 1..3 commits per day. When there are fresh commits I know it's time to restart the miner.
2. CTRL-C to stop `loda-rust mine` if it's running.
3. Pull the latest commits from the `loda-programs` repo and when resolving merge conflicts then choose "use theirs".
4. Run `loda-rust mine` again.
5. Wait approx 24 hours until there are fresh commits to the official `loda-programs` repo.

### Inner workings of `loda-rust mine`

While `loda-rust` is running, the found candiate programs are saved in the dir: `~/.loda-rust/mine-event`.

After 10 candidate programs have been accumulated, the attention switches from `mine` to `postmine`.
Here `loda-rust` determines: Does the program compute the correct terms, is the program faster than the existing program.
Underneeth `loda-rust` uses `loda-cpp`.
The `~/.loda-rust/postmine/19840101-010101-postmine` holds info about how the decisions was made.

The discovered programs are uploaded to the `loda-lang.org` server and scheduled for further processing on the server.
If the server determines that it's a new program or an improvement to an existing program, then it gets added to the official `loda-programs` repo.
In approx 24 hours, it shows up next time when fetching the `loda-programs` repo.

Also `loda-rust` places the programs inside the local `loda-programs` repo, but it doesn't do any commit.

### Additional steps in my daily routine

1. I manually inspect all diffs in my local `loda-programs` repo. 
2. If a single line has been inserted ala `mod $0,12345`, then I discard it.
3. Has magic constants been introduced, then I discard it.
4. Then commit it with the commit message: `Updated programs`.

When I have handwritten programs that that I want to upload to the server, then I run the script
```
PROMPT> cd loda-rust/script
PROMPT> ruby upload_program_files_to_server_from_commit.rb
snip .. takes about 20 seconds for 200 programs .. snip
PROMPT>
```
