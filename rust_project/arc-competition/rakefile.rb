desc "Create a docker image with the LODA-RUST executable inside"
task :build do
    # Go to parent dir is ugly. Why do this?
    # This is a workaround, since the Dockerfile cannot `COPY` files from the parent dir.
    # The Dockerfile has to be invoked within the dir where the files live.
    Dir.chdir("..") do
        `docker buildx use my_loda_builder`
        `docker buildx build --platform linux/amd64,linux/arm64 -t neoneye/loda-rust-cli:latest --push . -f arc-competition/arc.Dockerfile`
    end
end

task :default do
    system 'rake -T'
end
