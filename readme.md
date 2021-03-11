# LODA Lab

Hi, I'm Simon Strandgaard. I'm a fan of the [On-Line Encyclopedia of Integer Sequences® (OEIS®)](http://oeis.org/) and a fan of AI. 
This is my attempt at doing a crossover of both. This is experimental stuff.

This repo is inspired by [Christian Krause's LODA project](https://github.com/ckrause/loda) for mining integer sequences.
LODA has proved to be remarkable good at making programs that correspond to OEIS integer sequences.


# Usage - Basics

### Print 100 terms

Evaluate program for the A000079 oeis sequence, printing 100 terms.

```
PROMPT> cargo run -- eval 79 -t 10
1,2,4,8,16,32,64,128,256,512
PROMPT>
```

### Print dependencies

Print dependencies of a program for the A000073 oeis sequence.

```
PROMPT> cargo run -- deps 73
73,232508,301657
PROMPT>
```

# Usage - Advanced

### Run tests

```
PROMPT> cargo test
```


### Run with backtrace enabled

This is useful if there is a crash.

```
PROMPT> RUST_BACKTRACE=1 cargo run -- eval 79 -t 64
```

### Run and print instructions

Evaluate program for the A000079 oeis sequence, processing 2 terms and printing the instructions.

```
PROMPT> cargo run -- eval 79 -t 2 --instructions
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

