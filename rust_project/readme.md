# Content of the Rust Project dir

LODA Lab is written in the language Rust.



## Development

### Run tests

```
PROMPT> cargo test
```


### Run with debug output

```
PROMPT> RUST_LOG=DEBUG cargo run -- eval 40 -t 80
[2021-03-30T03:43:20Z DEBUG loda_lab::control::dependency_manager] program_id: 40  depends on other programs: [10051]
[2021-03-30T03:43:20Z DEBUG loda_lab::execute::node_call] NodeCall: update_call. program_id: 10051
2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71,73,79,83,89,97,101,103,107,109,113,127,131,137,139,149,151,157,163,167,173,179,181,191,193,197,199,211,223,227,229,233,239,241,251,257,263,269,271,277,281,283,293,307,311,313,317,331,337,347,349,353,359,367,373,379,383,389,397,401,409
[2021-03-30T03:43:21Z DEBUG loda_lab::control::subcommand_evaluate] steps: 3425789
[2021-03-30T03:43:21Z DEBUG loda_lab::control::subcommand_evaluate] cache: hit:14367 miss:490,0
[2021-03-30T03:43:21Z DEBUG loda_lab::control::subcommand_evaluate] elapsed: 916 ms
```


### Run with backtrace enabled

This is useful if there is a crash.

```
PROMPT> RUST_BACKTRACE=1 cargo run -- eval 79 -t 64
```


### Compile for release and run miner

```
PROMPT> cargo build -p loda-rust-cli --release
PROMPT> ./target/release/loda-rust mine
```


### Deploy for web

```
PROMPT> cd lodalab-web
PROMPT> wasm-pack build --target web
```

Open "index.html" in the browser.



