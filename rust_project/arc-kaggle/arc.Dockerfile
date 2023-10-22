####################################################################################################
## Builder
####################################################################################################
FROM rust:1.71.1 AS builder
WORKDIR /my_builddir
#COPY ./loda-rust-core loda-rust-core
#COPY ./loda-rust-cli loda-rust-cli
#COPY ./loda-rust-web loda-rust-web
COPY ./arc-kaggle arc-kaggle
RUN cargo build --config net.git-fetch-with-cli=true --release -p arc-kaggle

####################################################################################################
## Final image
####################################################################################################
FROM debian:bullseye-slim
COPY --from=builder /my_builddir/target/release/arc-kaggle /usr/local/bin/arc-kaggle
#COPY ./arc-competition/payload /root
#CMD ["loda-rust", "arc-competition"]
CMD ["arc-kaggle"]
