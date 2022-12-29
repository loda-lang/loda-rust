####################################################################################################
## Builder
####################################################################################################
#FROM rust:1.65.0 AS builder
#FROM --platform=linux/amd64 rust:1.65.0 AS builder
FROM amd64/rust:1.65.0 AS builder

#RUN rustup override set active-toolchain

#RUN rustup target add x86_64-unknown-linux-musl
#RUN rustup target remove aarch64-unknown-linux-gnu
#ENV RUSTUP_TOOLCHAIN=x86_64-unknown-linux-musl
#RUN rustup set default-host x86_64-unknown-linux-musl
#RUN rustup target list
#RUN rustup show active-toolchain

RUN apt update && apt install -y musl-tools musl-dev
RUN apt-get install -y build-essential
RUN yes | apt install gcc-x86-64-linux-gnu

# Create user
ENV USER=my-user
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /my_builddir

COPY ./ .

#ENV RUSTFLAGS='-C linker=x86_64-linux-gnu-gcc'
#ENV RUSTFLAGS='"-Clink-self-contained=on" -C linker=x86_64-linux-gnu-gcc'

#RUN rustup show active-toolchain
#CMD ["rustup show active-toolchain"]

# RUN cargo build --target x86_64-unknown-linux-musl --release -p loda-rust-cli
RUN cargo build --release -p loda-rust-cli
 
# ####################################################################################################
# ## Final image
# ####################################################################################################
# FROM scratch
# 
# # Import from builder.
# COPY --from=builder /etc/passwd /etc/passwd
# COPY --from=builder /etc/group /etc/group
# 
# WORKDIR /my_workdir
# 
# # Copy our build
# COPY --from=builder /my_builddir/target/x86_64-unknown-linux-musl/release/loda-rust-cli ./
# 
# # Use an unprivileged user.
# USER my-user:my-user
# 
# CMD ["/my_workdir/loda-rust-cli"]

