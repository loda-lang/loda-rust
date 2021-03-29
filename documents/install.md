# Installation guide for LODA Lab


---
## Step 1 - Install dependency

The `LODA Lab` project depends on the `LODA` project. So first install LODA.

#### Step 1 A

Check out [Christian Krause's LODA repository](https://github.com/ckrause/loda) on your computer.

A good place for this repository, is the `$HOME/git/loda` dir.

#### Step 1 B

Follow the `LODA` project install instructions.

LODA creates a `$HOME/.loda` dir.

#### Step 1 Complete

So far so good. LODA is installed.

---

## Step 2 - Install LODA Lab

#### Step 2 A

Check out [Simon Strandgaard's LODA Lab repository](https://github.com/neoneye/loda-lab) on your computer.

A good place for this repository, is the `$HOME/git/loda-lab` dir.

#### Step 2 B

Compile the `rust_project` into an executable named `loda-lab`.

```
PROMPT> pwd
/Users/JOHNDOE/git/loda-lab/rust_project
PROMPT> cargo build --release
PROMPT> cp target/release/loda-lab ..
```

#### Step 2 C

```
PROMPT> loda-lab install
```

This creates a `$HOME/.loda-lab` dir.

#### Step 2 D

Manually edit the configuration file `$HOME/.loda-lab/config.toml`.

Here you must update the paths, so they refer to where LODA is installed on your computer.

Manually modify this parameter, so it points to the dir that contains all the LODA programs.
```
loda_program_rootdir = "/Users/JOHNDOE/git/loda/programs/oeis"
```

Manually modify this parameter, so it points to the unzipped OEIS stripped file.
```
oeis_stripped_file = "/Users/JOHNDOE/.loda/oeis/stripped"
```

Manually modify this parameter, so it points to the LODA Lab repository dir.
```
loda_lab_repository = "/Users/JOHNDOE/git/loda-lab"
```

#### Step 2 E

Verify that LODA Lab really works, by computing [A000040, The prime numbers](https://oeis.org/A000040).

```
PROMPT> loda-lab eval 40
2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71
PROMPT>
```

#### Step 2 Complete

Finally `LODA Lab` is fully installed.

