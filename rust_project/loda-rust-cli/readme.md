# Command line interface for LODA

# Installation

It can be installed on macOS, Linux.

See [Installation Guide](https://github.com/loda-lang/loda-rust/blob/develop/documents/install.md).


# Usage

### Print 10 terms

Evaluate program for the A000079 oeis sequence, printing 10 terms.

```
PROMPT> loda-rust eval 79 -t 10
1,2,4,8,16,32,64,128,256,512
PROMPT>
```

### Print dependencies

Print dependencies of a program for the A000073 oeis sequence.

```
PROMPT> loda-rust deps 73
73,232508,301657
PROMPT>
```

### Print internal state

Evaluate program for the A000079 oeis sequence, processing 2 terms and printing the internal state.

```
PROMPT> loda-rust eval 79 -t 2 --debug
INPUT: a(0)
mov $1,2     [0,0] => [0,2]
pow $1,$0    [0,2] => [0,1]
OUTPUT: a(0) = 1
INPUT: a(1)
mov $1,2     [1,0] => [1,2]
pow $1,$0    [1,2] => [1,2]
OUTPUT: a(1) = 2
PROMPT>
```

### Identify patterns

Download the latest loda programs.

```bash
PROMPT> cd git/loda-cpp 
PROMPT> sh install.sh 
PROMPT> loda update
2024-08-27 15:46:24|INFO |Starting LODA v24.8.23. See https://loda-lang.org/
2024-08-27 15:46:24|INFO |Using LODA home directory "/Users/neoneye/loda/"
2024-08-27 15:46:25|INFO |Fetched https://raw.githubusercontent.com/loda-lang/loda-cpp/main/miners.default.json
2024-08-27 15:46:25|INFO |Updating OEIS index (last update 412 days ago)
2024-08-27 15:46:27|INFO |Fetched http://api.loda-lang.org/miner/v1/oeis/stripped.gz
2024-08-27 15:46:27|INFO |Fetched http://api.loda-lang.org/miner/v1/oeis/names.gz
2024-08-27 15:46:27|INFO |Updating programs repository (last update 412 days ago)
Updating files: 100% (81601/81601), done.
2024-08-27 15:46:46|INFO |Cleaning up local programs directory
2024-08-27 15:46:46|INFO |Removed 1 old local programs
2024-08-27 15:46:46|INFO |Loading sequences from the OEIS index
2024-08-27 15:46:48|INFO |Loaded 363020/375454 sequences in 2.41s
2024-08-27 15:46:48|INFO |Regenerating program stats (last update 412 days ago)
2024-08-27 15:46:59|INFO |Finished stats generation for 126861 programs
2024-08-27 15:47:00|INFO |Generating program lists at "/Users/neoneye/loda/lists/"
2024-08-27 15:47:00|INFO |Finished generation of lists for 126861 programs
```

What are the [`loda-patterns` repo](https://github.com/neoneye/loda-patterns) on my computer:

```bash
PROMPT> pwd
/Users/neoneye/git/loda-patterns
PROMPT> ls -1
LICENSE
README.md
simple_constant
```

What does my config look like:

```bash
PROMPT> cat ~/.loda-rust/config.toml 
loda_submitted_by = "Simon Strandgaard (M1)"

#miner_sync_executable = "$HOME/git/loda-rust/script/miner_sync_advanced.rb"

[miner_cpu_strategy]
type = "cpu"
[miner_cpu_strategy.content]
#count = 5
#count = 1
#count = 7
#count = 8
count = 10


#[miner_filter_mode]
#type = "all"
```

Identify the patterns

```bash
PROMPT> cargo build -p loda-rust-cli --release
PROMPT> ./target/release/loda-rust pattern --verbose
./target/release/loda-rust pattern
[2024-08-27T13:55:25Z ERROR loda_rust::subcommand_pattern] load program, error: ParseParameters(UnrecognizedParameterValue(26)) path: "/Users/neoneye/loda/programs/oeis/037/A037017.asm"
[2024-08-27T13:55:25Z ERROR loda_rust::subcommand_pattern] analyze_program. Skipping a program that is too long. path: "/Users/neoneye/loda/programs/oeis/058/A058319.asm"
[2024-08-27T13:55:25Z ERROR loda_rust::subcommand_pattern] load program, error: ParseParameters(UnrecognizedParameterValue(20)) path: "/Users/neoneye/loda/programs/oeis/069/A069858.asm"
[2024-08-27T13:55:26Z ERROR loda_rust::subcommand_pattern] load program, error: ParseParameters(UnrecognizedParameterValue(20)) path: "/Users/neoneye/loda/programs/oeis/113/A113851.asm"
[2024-08-27T13:55:26Z ERROR loda_rust::subcommand_pattern] load program, error: ParseParameters(UnrecognizedParameterValue(9)) path: "/Users/neoneye/loda/programs/oeis/121/A121977.asm"
[2024-08-27T13:55:26Z ERROR loda_rust::subcommand_pattern] analyze_program. Skipping a program that is too long. path: "/Users/neoneye/loda/programs/oeis/138/A138298.asm"
[2024-08-27T13:55:26Z ERROR loda_rust::subcommand_pattern] load program, error: ParseParameters(UnrecognizedParameterValue(19)) path: "/Users/neoneye/loda/programs/oeis/140/A140922.asm"
[2024-08-27T13:55:26Z ERROR loda_rust::subcommand_pattern] load program, error: ParseParameters(UnrecognizedParameterValue(19)) path: "/Users/neoneye/loda/programs/oeis/140/A140938.asm"
[2024-08-27T13:55:26Z ERROR loda_rust::subcommand_pattern] load program, error: ParseParameters(UnrecognizedParameterValue(19)) path: "/Users/neoneye/loda/programs/oeis/140/A140939.asm"
[2024-08-27T13:55:26Z ERROR loda_rust::subcommand_pattern] load program, error: ParseParameters(UnrecognizedParameterValue(12)) path: "/Users/neoneye/loda/programs/oeis/157/A157983.asm"
[2024-08-27T13:55:27Z ERROR loda_rust::subcommand_pattern] analyze_program. Skipping a program that is too long. path: "/Users/neoneye/loda/programs/oeis/245/A245425.asm"
[2024-08-27T13:55:27Z ERROR loda_rust::subcommand_pattern] analyze_program. Skipping a program that is too long. path: "/Users/neoneye/loda/programs/oeis/245/A245430.asm"
[2024-08-27T13:55:27Z ERROR loda_rust::subcommand_pattern] load program, error: ParseParameters(UnrecognizedParameterValue(23)) path: "/Users/neoneye/loda/programs/oeis/250/A250600.asm"
[2024-08-27T13:55:27Z ERROR loda_rust::subcommand_pattern] load program, error: ParseParameters(UnrecognizedParameterValue(23)) path: "/Users/neoneye/loda/programs/oeis/250/A250864.asm"
line count: 1  number of programs: 295
number of patterns: 6
line count: 2  number of programs: 3487
number of patterns: 25
line count: 3  number of programs: 4483
number of patterns: 39
line count: 4  number of programs: 4833
number of patterns: 37
line count: 5  number of programs: 5387
number of patterns: 30
line count: 6  number of programs: 5509
number of patterns: 30
line count: 55  number of programs: 2
number of patterns: 0
line count: 56  number of programs: 1
number of patterns: 0
line count: 57  number of programs: 4
number of patterns: 0
line count: 58  number of programs: 2
number of patterns: 0
line count: 60  number of programs: 3
number of patterns: 0
line count: 61  number of programs: 1
number of patterns: 0
line count: 62  number of programs: 2
number of patterns: 0
line count: 63  number of programs: 1
number of patterns: 0
line count: 65  number of programs: 1
number of patterns: 0
line count: 69  number of programs: 3
number of patterns: 0
line count: 72  number of programs: 2
number of patterns: 0
line count: 76  number of programs: 1
number of patterns: 0
elapsed: 19439 ms
```

Now `$HOME/git/loda-patterns` contains the identified patterns.
