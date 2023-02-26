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
PROMP> rake
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

### Step 1 - Delete old buildx instance

```
PROMP> rake remove-buildx-instance
```

### Step 2 - Populate payload directory

This is the data that is stored inside the docker image, such as program files, analytics data.

```
PROMP> rake payload
```

### Step 3 - Create buildx instance

In order to cross compile for multiple architectures.

```
PROMP> rake buildx-create
```

### Step 4 - Create the docker image

This takes around 12 minutes to compile!

```
PROMP> rake buildx-build
```

### Step 5 - Save the docker image to a tar file

```
PROMP> rake save-tar
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

### Step 6 - Run the docker image and see if it works

Manually copy around 60 json files from `ARC/data/training` to `secret_data/training`.

Check that this amount of json files are roughly also what is shows up when running the executable.

```
PROMP> rake run
1984-01-01T12:06:54Z - Start of program
initial program_item_vec: 66
initial model_item_vec.len: 63
snip output
CTRL-C to abort
```

Great this looks like the content of the `secret_data` has been mounted correct and the file has been discovered correct.

Now the `.tar` can be uploaded to the contest.

