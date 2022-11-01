# Installation guide for "loda-rust"

## Prerequisites

Computer with 64 bit cpu.


## Step 1 - Install LODA-CPP

Follow the [Install guide for LODA](https://loda-lang.org/install/).

Verify that LODA-CPP really works

```
PROMPT> loda eval A40
2,3,5,7,11,13,17,19,23,29
```


## Step 2 - Rust language

On macOS/Linux you may want to use your package manager.

Otherwise here is the [Install guide for Rust](https://www.rust-lang.org/learn/get-started).

Verify that Rust really works

```
PROMPT> cargo --version
cargo 1.63.0 (fd9c4297c 2022-07-01)
```


## Step 3 - Obtain "loda-rust" repository

#### Step 3 A

Check out [loda-rust](https://github.com/loda-lang/loda-rust) on your computer.

A good place for this repository, is the `$HOME/git/loda-rust` dir.

#### Step 3 B

Compile the `rust_project` into an executable named `loda-rust`.

```
PROMPT> pwd
/Users/JOHNDOE/git/loda-rust/rust_project
PROMPT> cargo build --release
PROMPT> ./target/release/loda-rust
loda-rust 0.0.1
Experimental tool

USAGE:
    loda-rust <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    analytics       Prepare data needed for mining, by analyzing the existing programs.
    dependencies    Print all direct/indirect dependencies of a program
    evaluate        Evaluate a program
    help            Print this message or the help of the given subcommand(s)
    install         Create the $HOME/.loda-rust directory
    mine            Run the miner daemon process. Press CTRL-C to stop it.
    pattern         Identify recurring patterns among similar programs.
    postmine        Validate the accumulated candiate programs for correctness and performance.
    similar         Identify similar programs.
PROMPT>
```

#### Step 3 C

On linux/macOS: Create symlink to the executable from within a `bin` dir, so `loda-rust` is available in `$PATH`.

```
PROMPT> cd ~/bin
PROMPT> ln -s ~/git/loda-rust/rust_project/target/release/loda-rust
PROMPT>
```

Check that `loda-rust` is still available, like this:

```
PROMPT> loda-rust
```

#### Step 3 D

```
PROMPT> loda-rust install
```

This creates a `$HOME/.loda-rust` dir.

#### Step 3 E

Manually edit the configuration file `$HOME/.loda-rust/config.toml`.

Here you must update the paths, so they refer to where LODA-CPP is installed on your computer.

Manually modify this parameter, so it points to the "loda-programs" repository dir.
```
loda_programs_repository = "/Users/JOHNDOE/loda/programs"
```

Manually modify this parameter, so it points to the LODA-RUST repository dir.
```
loda_rust_repository = "/Users/JOHNDOE/git/loda-rust"
```

Manually modify this parameter, so it points to the unzipped OEIS stripped file.
```
oeis_stripped_file = "/Users/JOHNDOE/loda/oeis/stripped"
```

#### Step 3 F

Verify that "loda-rust" really works, by computing [A000040, The prime numbers](https://oeis.org/A000040).

```
PROMPT> loda-rust eval 40
2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71
PROMPT>
```

#### Step 3 Complete

Finally `loda-rust` is fully installed.

