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
rake buildx-build   # Create a docker image with the LODA-RUST executable inside
rake buildx-create  # Create a docker builder named 'my_loda_builder'
rake payload        # Prepare the contents of the /root dir
rake run            # Runs the LODA-RUST process in an isolated container, in an enviroment similar to the ARCathon submission specification
rake shell          # Runs an interactive bash shell inside the docker image, so it's possible to troubleshoot
```

The `payload` dir is copied into the `/root` dir inside the docker image, which is the `$HOME` dir.

The `secret_data` dir is mounted at `/data` inside the docker image. This is how the docker image communicates with the outside world.

The `/data/solution/solution_notXORdinary.json` contains the predicted solutions. While the puzzle solver code is running, it continuously updates this file with its findings.

