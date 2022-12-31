docker buildx use my_loda_builder
docker buildx build --platform linux/amd64,linux/arm64 -t neoneye/loda-rust-cli:latest --push . -f arc.Dockerfile

