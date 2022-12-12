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
