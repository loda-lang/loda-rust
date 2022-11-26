# LODA-RUST - Get the miner running

Warning: `LODA-RUST` mining is highly experimental.

Question: Can `loda-rust mine` run by itself?

Answer: No, currently I’m doing some steps manually. Over time I want to automate it.


## Usage - simple

The following starts the miner, that continues until it gets killed by CTRL-C.

Look for the `miner discovered a "new" program. A257071` row.

```
PROMPT> loda-rust mine
[snip]
candidate: "20221101-155518-4398841.asm"
candidate: "20221101-155529-4460404.asm"
candidate: "20221101-155548-4572041.asm"
candidate: "20221101-155553-4504272.asm"
trigger start postmine
postmine_worker: child d181d52b-ab06-4a36-aa81-6d13e2dc2259, received broadcast message: StartPostmineJob
BEFORE PostMine::run()
Ignoring 9999 programs that have already been analyzed
Number of pending programs: 10
Arrange programs by priority. high prio: 10, low prio: 0
    Finished Ran loda-cpp with pending programs, in 0 seconds
Looking up in the OEIS 'stripped' file
    Finished Lookups in the OEIS 'stripped' file, in 2 seconds
Minimizing programs
    Finished Minimized programs, in 2 seconds
Looking up in the OEIS 'names' file
    Finished Lookups in the OEIS 'names' file, in 0 seconds
Analyzing 14 program ids
miner discovered a "new" program. A257071             <----------- This is when a new program has been found
    Finished Analyzed pending programs, in 3 minutes
AFTER PostMine::run()
postmine Ok
trigger resume mining
candidate: "20221101-155853-4590837.asm"
candidate: "20221101-155935-4666859.asm"
candidate: "20221101-160018-4724486.asm"
candidate: "20221101-160032-4813221.asm"
candidate: "20221101-160253-5277447.asm"
candidate: "20221101-160407-5418834.asm"
[snip]
PROMPT>
```

Check if the miner has crashed, and write a bug report to `neoneye@gmail.com`.

Wait approx 24 hours until there are fresh commits to the official `loda-programs` repo.

Look at the slack channel to see live what programs have been mined.

## Usage with metrics on dashboard

Install dashboard [grafana](https://grafana.com/).

Install the time series database [prometheus](https://prometheus.io/).

Launch the miner like this: `loda-rust mine --metrics`. 

The `--metrics` makes the metrics stats available at `http://localhost:8090/metrics`.
Place the [prometheus.yml](https://github.com/loda-lang/loda-rust/blob/develop/resources/realtime%20metrics/prometheus.yml) in the same dir as prometheus.

Inside grafana's load the [grafana-dashboard.json](https://github.com/loda-lang/loda-rust/blob/develop/resources/realtime%20metrics/grafana-dashboard.json).


## Inner workings of `loda-rust mine`

While `loda-rust` is running, the found candiate programs are saved in the dir: `~/.loda-rust/mine-event`.

After several candidate programs have been accumulated, the attention switches from `mine` to `postmine`.
Here `loda-rust` determines: Does the program compute the correct terms, is the program faster than the existing program.
Underneath `loda-rust` uses `loda-cpp`.
The `~/.loda-rust/postmine/19840101-010101-postmine` holds info about how the decisions was made.

The discovered programs are uploaded to the `loda-lang.org` server and scheduled for further processing on the server.
If the server determines that it's a new program or an improvement to an existing program, then it gets added to the official `loda-programs` repo.
In approx 24 hours, it shows up next time when fetching the `loda-programs` repo.

The discovered programs are placed inside the local `loda-programs` repo.

The `miner_sync_executable` is executed every hour (or some interval). If there are no new commits in the official [loda-programs](https://github.com/loda-lang/loda-programs) repo, then nothing happens.
When there is a new commit to official loda-programs repo, then it syncs the local repository with it.

---

# Config file

## What does my config file look like?

Hi I'm Simon Strandgaard, and is a developer working on LODA-RUST, and this is my config.

The file `~/.loda-rust/config.toml`

```
loda_submitted_by = "Simon Strandgaard"
miner_sync_executable = "$HOME/git/loda-rust/script/miner_sync_advanced.rb"

[miner_cpu_strategy]
type = "cpu"
[miner_cpu_strategy.content]
count = 5
```

I use 5 cpus for mining. So that I can use my computer for other things.
Sometimes I comment out this section, and then I'm using all cpus for mining, 

I use the `miner_sync_advanced.rb` which requires several steps of setup.


## What should my config file look like?

If you want to mine for new programs, and you are not a LODA developer. Then this is what the config file should look like.

The file `~/.loda-rust/config.toml`

```
loda_submitted_by = "Your Name"
```

The available settings and their default values, are listed here: [default_config.toml](https://github.com/loda-lang/loda-rust/blob/develop/rust_project/loda-rust-cli/src/config/default_config.toml).


---

# Misc tasks

## Task: Upload manually coded LODA program to loda-lang

I sometimes have a handwritten program that that I want to upload to the server.

Approach A:
I save the program in `~/.loda-rust/mine-event/my-handwritten-program.asm` and run `loda-rust mine`.
This checks the program is correct, does code formatting. Should there be an existing program, then it compares performance.
Drawback: It strips manual written comments from the program.

Approach B:
When I have handwritten programs that that I want to upload to the server, then I run the script.
Benefit: It preserves the comments.
Drawback: It doesn't insert sequence names for `seq` instructions.
```
PROMPT> cd loda-rust/script
PROMPT> ruby upload_program_files_to_server_from_commit.rb
snip .. takes about 20 seconds for 200 programs .. snip
PROMPT>
```

