require 'time'

# The docker image can be pulled from here:
# https://hub.docker.com/r/neoneye/loda-rust-cli/tags
DOCKER_IMAGE = "neoneye/loda-rust-cli:latest"

desc "Create a docker builder named 'my_loda_builder'"
task 'buildx-create' do
    system("docker buildx create --platform linux/amd64 --name my_loda_builder")
end

desc "Prepare the contents of the /root dir. This is to be invoked before 'buildx-build'."
task "payload" do
    system("rsync -a --delete --exclude='.git/' ~/git/loda-arc-challenge/ payload/loda-arc-challenge")
    system("rsync -a --delete ~/.loda-rust/analytics-arc/ payload/.loda-rust/analytics-arc")
end

desc "Create a docker image with the LODA-RUST executable inside"
task 'buildx-build' do
    image = DOCKER_IMAGE
    # Go to parent dir is ugly. Why do this?
    # This is a workaround, since the Dockerfile cannot `COPY` files from the parent dir.
    # The Dockerfile has to be invoked within the dir where the files live.
    Dir.chdir("..") do
        system("docker buildx use my_loda_builder")
        system("docker buildx build --platform linux/amd64 -t #{image} --push . -f arc-competition/arc.Dockerfile")
    end
end

desc "Runs the LODA-RUST process in an isolated container, in an enviroment similar to the ARCathon submission specification."
task "run" do
    pwd = Dir.pwd
    image = DOCKER_IMAGE
    system("docker run --platform linux/amd64 --mount type=bind,source=#{pwd}/secret_data,target=/data #{image}")
end

desc "Runs an interactive bash shell inside the docker image, so it's possible to troubleshoot."
task "shell" do
    pwd = Dir.pwd
    image = DOCKER_IMAGE
    system("docker run --platform linux/amd64 -it --mount type=bind,source=#{pwd}/secret_data,target=/data #{image} /bin/bash")
end

desc "Export docker image to a tgz file"
task "save-tgz" do
    pwd = Dir.pwd
    image = DOCKER_IMAGE
    timestamp = Time.now.utc.iso8601
    filename = "docker_lodarust_arc_#{timestamp}.tgz" 
    system("docker pull --platform linux/amd64 #{image}")
    system("docker save #{image} | gzip > '#{filename}'")
end

desc "Remove the buildx instance - after building the docker image, it's no longer needed."
task "remove-buildx-instance" do
    buildxls = `docker buildx ls`
    if buildxls =~ /my_loda_builder/
        puts "will purge"
        system("docker buildx rm my_loda_builder")
        puts "did purge"
    else
        puts "nothing to be purged"
    end
end

task :default do
    system 'rake -T'
end
