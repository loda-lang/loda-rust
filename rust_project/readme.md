# Content of the `rust_project` dir

LODA Rust is written in the language Rust.

For development I use [VSCode](https://github.com/microsoft/vscode).

## Run tests

Before start doing development on LODA-RUST, make sure the tests runs without failures.

```
PROMPT> cargo test
```


## Run with debug output

LODA-RUST outputs things to console that are hidden by default.

In order to see it, prefix with `RUST_LOG=DEBUG`.

```
PROMPT> RUST_LOG=DEBUG cargo run -- eval A40
   Compiling loda-rust-web v0.1.0 (/Users/homedir/git/loda-rust/rust_project/loda-rust-web)
   Compiling loda-rust-cli v0.1.0 (/Users/homedir/git/loda-rust/rust_project/loda-rust-cli)
    Finished dev [unoptimized + debuginfo] target(s) in 6.43s
     Running `target/debug/loda-rust eval A40`
2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71
[1999-12-29T09:59:01Z DEBUG loda_rust::subcommand_evaluate] steps: 20390
[1999-12-29T09:59:01Z DEBUG loda_rust::subcommand_evaluate] cache: hit:190 miss:111,0
[1999-12-29T09:59:01Z DEBUG loda_rust::subcommand_evaluate] elapsed: 12 ms
```


## Run with backtrace enabled

This is useful if there is a crash.

```
PROMPT> RUST_BACKTRACE=1 cargo run -- eval A79
```


## Compile in release mode

```
PROMPT> cargo build -p loda-rust-cli --release
PROMPT> ./target/release/loda-rust eval A40
2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71
PROMPT>
```

## Deploy for web

```
PROMPT> cd loda-rust-web
PROMPT> ./build.sh
```

Open "index.html" in the browser.


## Verify integration with loda-cpp is working

```
PROMPT> cargo run -- test-integration-with-lodacpp
test integration with lodacpp: Completed successfully.
PROMPT>
```

