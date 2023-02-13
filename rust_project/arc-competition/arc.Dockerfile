####################################################################################################
## Builder
####################################################################################################
FROM --platform=linux/amd64 rust:1.65.0 AS builder
WORKDIR /my_builddir
COPY ./loda-rust-core loda-rust-core
COPY ./loda-rust-cli loda-rust-cli
COPY ./loda-rust-web loda-rust-web
COPY ./Cargo.toml Cargo.toml
RUN cargo build --release -p loda-rust-cli

####################################################################################################
## Final image
####################################################################################################
FROM --platform=linux/amd64 debian:bullseye-slim
COPY --from=builder /my_builddir/target/release/loda-rust /usr/local/bin/loda-rust
COPY ./arc-competition/payload /root
CMD ["loda-rust", "arc-competition"]
