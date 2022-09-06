# Installation guide for "loda-rust"

## Prerequisites

Computer with 64 bit cpu.

LODA-CPP doesn't run on Raspberry Pi 4 with armv7l cpu (32bit).


## Step 1 - Install LODA Cpp

Follow the [Install guide for LODA](https://loda-lang.org/install/).

Verify that LODA Cpp really works

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


## Step 3 - LODA Rust repository

#### Step 3 A

Check out [loda-rust](https://github.com/loda-lang/loda-rust) on your computer.

A good place for this repository, is the `$HOME/git/loda-rust` dir.

#### Step 3 B

Compile the `rust_project` into an executable named `loda-rust`.

```
PROMPT> pwd
/Users/JOHNDOE/git/loda-rust/rust_project
PROMPT> cargo build --release
PROMPT> cp target/release/loda-rust ..
```

#### Step 3 C

```
PROMPT> loda-rust install
```

This creates a `$HOME/.loda-rust` dir.

#### Step 3 D

Manually edit the configuration file `$HOME/.loda-rust/config.toml`.

Here you must update the paths, so they refer to where LODA Cpp is installed on your computer.

Manually modify this parameter, so it points to the "loda-programs" repository dir.
```
loda_programs_repository = "/Users/JOHNDOE/loda/programs"
```

Manually modify this parameter, so it points to the LODA Rust repository dir.
```
loda_rust_repository = "/Users/JOHNDOE/git/loda-rust"
```

Manually modify this parameter, so it points to the unzipped OEIS stripped file.
```
oeis_stripped_file = "/Users/JOHNDOE/loda/oeis/stripped"
```

#### Step 3 E

Verify that "loda-rust" really works, by computing [A000040, The prime numbers](https://oeis.org/A000040).

```
PROMPT> loda-rust eval 40
2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71
PROMPT>
```

#### Step 3 Complete

Finally `loda-rust` is fully installed.

