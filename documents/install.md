# Installation guide for LODA Lab

## Step 1 - Rust language

On macOS/Linux you may want to use your package manager.

Otherwise here is the [Install guide for Rust](https://www.rust-lang.org/learn/get-started).

Verify that Rust really works

```
PROMPT> cargo --version
cargo 1.48.0 (65cbdd2dc 1984-12-29)
```


## Step 2 - LODA repository

The `LODA Lab` project depends on the `LODA` project. So first install LODA.

#### Step 2 A

Check out [loda-programs](https://github.com/loda-lang/loda-programs) on your computer.

A good place for this repository, is the `$HOME/git/loda-cpp` dir.

#### Step 2 B

Check out [loda-cpp](https://github.com/loda-lang/loda-cpp) on your computer.

A good place for this repository, is the `$HOME/git/loda-cpp` dir.

#### Step 2 C

Follow the `loda-cpp` project install instructions.

LODA creates a `$HOME/.loda` dir.

#### Step 2 Complete

So far so good. LODA is installed.



## Step 3 - LODA Rust repository

#### Step 3 A

Check out [LODA Rust repository](https://github.com/loda-lang/loda-rust) on your computer.

A good place for this repository, is the `$HOME/git/loda-rust` dir.

#### Step 3 B

Compile the `rust_project` into an executable named `loda-lab`.

```
PROMPT> pwd
/Users/JOHNDOE/git/loda-rust/rust_project
PROMPT> cargo build --release
PROMPT> cp target/release/loda-lab ..
```

#### Step 3 C

```
PROMPT> loda-lab install
```

This creates a `$HOME/.loda-rust` dir.

#### Step 3 D

Manually edit the configuration file `$HOME/.loda-rust/config.toml`.

Here you must update the paths, so they refer to where LODA is installed on your computer.

Manually modify this parameter, so it points to the dir that contains all the LODA programs.
```
loda_program_rootdir = "/Users/JOHNDOE/git/loda-programs/oeis"
```

Manually modify this parameter, so it points to the unzipped OEIS stripped file.
```
oeis_stripped_file = "/Users/JOHNDOE/.loda/oeis/stripped"
```

Manually modify this parameter, so it points to the LODA Rust repository dir.
```
loda_rust_repository = "/Users/JOHNDOE/git/loda-rust"
```

Manually modify this parameter, so it points to the LODA Cpp repository dir.
```
loda_cpp_repository = "/Users/JOHNDOE/git/loda-cpp"
```

#### Step 3 E

Verify that LODA Lab really works, by computing [A000040, The prime numbers](https://oeis.org/A000040).

```
PROMPT> loda-lab eval 40
2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71
PROMPT>
```

#### Step 3 Complete

Finally `LODA Lab` is fully installed.

