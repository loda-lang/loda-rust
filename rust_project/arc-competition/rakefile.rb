# The docker image can be pulled from here:
# https://hub.docker.com/r/neoneye/loda-rust-cli/tags
DOCKER_IMAGE = "neoneye/loda-rust-cli:latest"

desc "Create a docker builder named 'my_loda_builder'"
task 'buildx-create' do
    system("docker buildx create --platform linux/arm64,linux/amd64 --name my_loda_builder")
end

desc "Prepare the contents of the /root dir. This is to be invoked before 'buildx-build'."
task "payload" do
    system("rsync -a --delete /Users/neoneye/git/loda-arc-challenge/ payload/loda-arc-challenge")
    system("rsync -a --delete /Users/neoneye/.loda-rust/analytics-arc/ payload/.loda-rust/analytics-arc")
end

desc "Create a docker image with the LODA-RUST executable inside"
task 'buildx-build' do
    image = DOCKER_IMAGE
    # Go to parent dir is ugly. Why do this?
    # This is a workaround, since the Dockerfile cannot `COPY` files from the parent dir.
    # The Dockerfile has to be invoked within the dir where the files live.
    Dir.chdir("..") do
        system("docker buildx use my_loda_builder")
        system("docker buildx build --platform linux/amd64,linux/arm64 -t #{image} --push . -f arc-competition/arc.Dockerfile")
    end
end

desc "Runs the LODA-RUST process in an isolated container, in an enviroment similar to the ARCathon submission specification."
task "run" do
    pwd = Dir.pwd
    image = DOCKER_IMAGE
    system("docker run --mount type=bind,source=#{pwd}/secret_data,target=/data #{image}")
end

desc "Runs an interactive bash shell inside the docker image, so it's possible to troubleshoot."
task "shell" do
    pwd = Dir.pwd
    image = DOCKER_IMAGE
    system("docker run -it --mount type=bind,source=#{pwd}/secret_data,target=/data #{image} /bin/bash")
end

task :default do
    system 'rake -T'
end
