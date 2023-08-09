# ARC is Abstraction and Reasoning Corpus

LODA-RUST solves 6 out of 100 hidden tasks in ARC. A human have no problems in solving these tasks.

As of 2023 ARC is unsolved.

ARC differs from OEIS in these ways:
- A LODA program for OEIS takes 1 input and returns 1 output. 
- A LODA program for ARC takes multiple inputs and returns multiple outputs.
- The way of mutating programs is the same for OEIS and ARC.
- The ARC code has lots of code for doing image manipulation.

# What is ARC?

[Explanation video](https://youtu.be/rLGpNcQ5alI).

Try solve a few of the easy puzzles
https://volotat.github.io/ARC-Game/

For a human the easy puzzles are somewhat easy to solve.

For a computer this collection of puzzles is hard.

The ARC 1 dataset is available here:
https://github.com/fchollet/ARC

# ARC-WEB

Loda-rust has a builtin webserver. Via this UI it's possible to inspect ARC tasks in great detail.

Manually edit the `Cargo.toml` file, and enable the feature `loda-rust-arc`:

```
[features]
default = ["loda-rust-arc"]
```

Start the webserver

```
PROMPT> cargo run -- arc-web
Open this manually in a browser: 127.0.0.1:8090/task
Press CTRL-C to stop the web server
```

Open `127.0.0.1:8090/task` in your browser. This shows a list of all the ARC tasks.

Let me know if there is something you need in the UI.
