# LODA-RUST - Insert histogram metadata into ARC task json files

The inserted histogram rows looks like this:

```json
{
  "metadata": {
    "histogram-az-space-newline-randomnewline triksupcwavg": "long markdown document with histogram comparisons",
    "histogram-AZ-none-comma ULRQKZVADCES": "long markdown document with histogram comparisons",
    "histogram-digit-none-comma 4301285967": "long markdown document with histogram comparisons",
  }
}
```

## Usage

### Step 1 - Enable ARC feature

Open the file `loda-rust-cli/Cargo.toml` in a text editor.

Enable the `loda-rust-arc` feature. By remove the dash in front of the line: `default = ["loda-rust-arc"]`

Save and close `loda-rust-cli/Cargo.toml`.


### Build an executable

```sh
PROMPT> cargo build --release -p loda-rust
```

### Run the executable

Provide the path to the directory containing ARC json files.

```sh
PROMPT> loda-rust arc-metadata-histograms --directory /tmp/dataset/ARC --count 100 --seed 42
```


## Developer

This runs slower. And prints out verbose details.

```sh
PROMPT> RUST_LOG=debug cargo run -- arc-metadata-histograms --count 1 --seed 42 --directory /tmp/dataset/ARC
```

