docker run -it --mount type=bind,source="$(pwd)"/secret_data,target=/data neoneye/loda-rust-cli:latest /bin/bash
