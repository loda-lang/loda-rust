# Content of the Rust Project dir

LODA Rust is written in the language Rust.



## Development

### Run tests

```
PROMPT> cargo test
```


### Run with debug output

```
PROMPT> RUST_LOG=DEBUG cargo run -- eval 40 -t 80
[2021-03-30T03:43:20Z DEBUG loda_rust::control::dependency_manager] program_id: 40  depends on other programs: [10051]
[2021-03-30T03:43:20Z DEBUG loda_rust::execute::node_call] NodeCall: update_call. program_id: 10051
2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71,73,79,83,89,97,101,103,107,109,113,127,131,137,139,149,151,157,163,167,173,179,181,191,193,197,199,211,223,227,229,233,239,241,251,257,263,269,271,277,281,283,293,307,311,313,317,331,337,347,349,353,359,367,373,379,383,389,397,401,409
[2021-03-30T03:43:21Z DEBUG loda_rust::control::subcommand_evaluate] steps: 3425789
[2021-03-30T03:43:21Z DEBUG loda_rust::control::subcommand_evaluate] cache: hit:14367 miss:490,0
[2021-03-30T03:43:21Z DEBUG loda_rust::control::subcommand_evaluate] elapsed: 916 ms
```


### Run with backtrace enabled

This is useful if there is a crash.

```
PROMPT> RUST_BACKTRACE=1 cargo run -- eval 79 -t 64
```


### Compile for release and run miner

```
PROMPT> cargo build -p loda-rust-cli --release
PROMPT> ./target/release/loda-rust analytics
PROMPT> ./target/release/loda-rust mine
press CTRL-C to stop
```

Let the miner run for an hour, and look at what gets accumulated inside the dir: `~/.loda-rust/mine-event`.

```
PROMPT> ls ~/.loda-rust/mine-event
20220726-043330-1120651120.asm
20220726-043416-1113532221.asm
20220726-044026-1131813741.asm
SNIP
20220726-045337-1188715008.asm
20220726-045934-1194346349.asm
20220726-050337-1211159643.asm
PROMPT> 
```

When there are 100 items in the dir `~/.loda-rust/mine-event`, then it's time for running `loda-rust postmine`.

```
PROMPT> ./target/release/loda-rust postmine
```


### Deploy for web

```
PROMPT> cd loda-rust-web
PROMPT> ./build.sh
```

Open "index.html" in the browser.


### Verify integration with loda-cpp is working

```
PROMPT> cargo run -- test-integration-with-lodacpp
test integration with lodacpp: Completed successfully.
PROMPT>
```

