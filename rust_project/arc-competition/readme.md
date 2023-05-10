# ARC competition

## What is ARC about?

The goal is to solve never-before-seen puzzles.

This is a contest that happens annually in December.

Each participant upload a docker image containing their puzzle solving code.

The contest organizers runs all the docker images.

Input/output is provided via the `/data` dir. Input is the puzzles. Output is the predicted solutions.

# What is the `arc-competition` dir?

This dir contain tools that automate constructing of a docker image.

```
PROMPT> rake
rake buildx-build            # Create a docker image with the LODA-RUST executable inside
rake buildx-create           # Create a docker builder named 'my_loda_builder'
rake payload                 # Prepare the contents of the /root dir
rake remove-buildx-instance  # Remove the buildx instance - after building the docker image, it's no longer needed
rake run                     # Runs the LODA-RUST process in an isolated container, in an enviroment similar to the ARCathon submission specification
rake save-tar                # Export docker image to tar file
rake shell                   # Runs an interactive bash shell inside the docker image, so it's possible to troubleshoot
```

The `payload` dir is copied into the `/root` dir inside the docker image, which is the `$HOME` dir.

The `secret_data` dir is mounted at `/data` inside the docker image. This is how the docker image communicates with the outside world.

The `/data/solution/solution_notXORdinary.json` contains the predicted solutions. While the puzzle solver code is running, it continuously updates this file with its findings.

# Deploy docker image

These are the steps to create a docker image and submit it to the contest.

### Step 1 - Bump version of LODA-RUST

In the `loda-rust-cli/Cargo.toml`

Insert todays date, like this: `version = "1984.12.31"`

Don't commit the file in this step. Instead commit after the docker container has been created and verified.


### Step 2 - Delete old docker containers and old docker images

Open the `Docker Desktop` app.

First manually remove old `containers`.

Secondly manually remove old `images`.

### Step 3 - Login to Docker

Open the `Docker Desktop` app.

Navigate to `Images -> REMOTE REPOSITORIES` and click `Sign in` to the docker account.

In the terminal app.

```
PROMPT> docker login
Authenticating with existing credentials...
Login Succeeded

Logging in with your password grants your terminal complete access to your account. 
For better security, log in with a limited-privilege personal access token. Learn more at https://docs.docker.com/go/access-tokens/
PROMPT>
```

### Step 4 - Delete old buildx instance

```
PROMPT> rake remove-buildx-instance
will purge
my_loda_builder removed
did purge
PROMPT>
```

### Step 5 - Populate payload directory

This is the data that is stored inside the docker image, such as program files, analytics data.

```
PROMPT> cargo build --release -p loda-rust-cli
PROMPT> loda-rust analytics-arc
PROMPT> rake payload
```

### Step 6 - Create buildx instance

In order to cross compile for multiple architectures.

```
PROMPT> rake buildx-create
```

### Step 7 - Create the docker image

This takes around 12 minutes to compile!

```
PROMPT> rake buildx-build
```

### Step 8 - Save the docker image to a tar file

```
PROMPT> rake save-tar
latest: Pulling from username/loda-rust-cli
bb263680fde1: Pull complete 
6055b99811ee: Pull complete 
db6ade30b079: Pull complete 
Digest: sha256:9c93f5982d4f85b8bc3e6b78fa4b39de4d04ac63b49bf9445bbdcddd7da61660
Status: Downloaded newer image for username/loda-rust-cli:latest
docker.io/username/loda-rust-cli:latest
PROMPT> ls -la
-rw-r--r--   1 neoneye  staff  101791744 Feb 26 13:03 docker_lodarust_arc_2023-02-26T12:03:03Z.tar
PROMPT>
```

### Step 9 - Run the docker image and see if it works

Manually copy around 60 json files from `ARC/data/training` to `secret_data/training`.

Check that this amount of json files are roughly also what is shows up when running the executable.

```
PROMPT> rake run
1984-01-01T12:06:54Z - Start of program
env::consts::ARCH: x86_64
env::consts::OS: linux
initial program_item_vec: 66
initial model_item_vec.len: 63
snip output
Press CTRL-C to stop it.
```

Verify that the architecture is `x86_64`. This is the only platform that is accepted for ARCathon.

Great this looks like the content of the `secret_data` has been mounted correct and the file has been discovered correct.

Now the `.tar` can be uploaded to the contest.

### Step 10 - Publish the docker image

Add the docker image `.tar` file to the [arcathon-docker-image](https://github.com/neoneye/arcathon-docker-image) repository.

After `git push` has finished.

Obtain the url for the docker image `.tar` file, that looks like this:

```
https://github.com/neoneye/arcathon-docker-image/raw/main/ARCathon2023/2023-02-26T13-03.tar
```

## Step 11 - Check that the docker image url actually downloads the file

Paste the docker image url into the browser.

Verify that a +100mb file gets downloaded.

Verify that the file can get loaded by docker:

```
PROMPT> docker load < 2023-02-26T13-03.tar
```

Verify that the docker image can run:

```
PROMPT> docker run --platform linux/amd64 --mount type=bind,source="$(pwd)"/secret_data,target=/data neoneye/loda-rust-cli:latest
prints out lots of stuff
Press CTRL-C to stop it.
```

We have verified that the url works, and that the docker image is runnable.

Delete the downloaded file again.

## Step 12 - Submission

Great. This docker image is ready to be submitted.

[ARCathon submission formula](https://lab42.global/arcathon/submission/)

