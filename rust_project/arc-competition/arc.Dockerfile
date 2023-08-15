####################################################################################################
## Builder
####################################################################################################
FROM rust:1.71.1 AS builder
WORKDIR /my_builddir
COPY ./loda-rust-core loda-rust-core
COPY ./loda-rust-cli loda-rust-cli
COPY ./loda-rust-web loda-rust-web
COPY ./Cargo.toml Cargo.toml
RUN cargo build --config net.git-fetch-with-cli=true --release -p loda-rust-cli

####################################################################################################
## Final image
####################################################################################################
FROM debian:bullseye-slim
COPY --from=builder /my_builddir/target/release/loda-rust /usr/local/bin/loda-rust
COPY ./arc-competition/payload /root
CMD ["loda-rust", "arc-competition"]
