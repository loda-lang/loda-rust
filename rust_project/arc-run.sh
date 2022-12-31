#docker run neoneye/loda-rust-cli:latest
docker run --mount type=bind,source="$(pwd)"/secret_data,target=/data neoneye/loda-rust-cli:latest
#docker run --mount type=bind,source="$(pwd)"/secret_data,target=/data -it neoneye/loda-rust-cli:latest /bin/bash
