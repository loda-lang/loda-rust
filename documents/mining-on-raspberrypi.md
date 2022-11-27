# LODA-RUST mining on a raspberrypi

Warning: `LODA-RUST` mining is highly experimental.

Please let me know about issues you may encounter, or ideas for improvement.


# Installing LODA-CPP - used by LODA-RUST

I'm following [this install guide](https://loda-lang.org/install/).

```
euclid@raspberrypi:~ $ /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/loda-lang/loda-cpp/main/install.sh)"
===== Welcome to LODA v22.11.22! =====

This command will guide you through its setup.

Enter the directory where LODA should store its files.
Press return for the default location (see below).
[/home/euclid/loda] 

We recommend to add the following line to your shell configuration:
export PATH=$PATH:/home/euclid/loda/bin
Do you want the setup to add it to /home/euclid/.bashrc? (Y/n) 

Done. Please run 'source /home/euclid/.bashrc' after this setup.
Press enter to continue the setup.


LODA needs to download its programs repository from GitHub.
The repository requires around 350 MB of disk space.
Checking whether git is installed:
git version 2.30.2

Press return to download the default programs repository:
[https://github.com/loda-lang/loda-programs.git] 
Cloning into '/home/euclid/loda/programs'...
remote: Enumerating objects: 754357, done.
remote: Counting objects: 100% (54482/54482), done.
remote: Compressing objects: 100% (38252/38252), done.
remote: Total 754357 (delta 18052), reused 44667 (delta 16224), pack-reused 699875
Receiving objects: 100% (754357/754357), 264.69 MiB | 2.82 MiB/s, done.
Resolving deltas: 100% (547351/547351), done.
Updating files: 100% (109788/109788), done.

LODA supports the following modes for mining programs:

1. Local Mode: mined programs are stored in your local
   programs folder only.

2. Client Mode (default): mined programs are stored in
   your local programs folder and also submitted to the
   central API server at https://loda-lang.org.

3. Server Mode: process submissions from the central API
   server and integrate them into the global programs
   repository.

Choose your mining mode:
[2] 

If you want to mine programs, LODA can automatically add
your name as a comment in the mined programs. If you specify
your name and run the miner in client mode, you give consent
to submit mined programs with your name and to publish them
at https://loda-lang.org and the programs repository at
https://github.com/loda-lang/loda-programs.

Enter your name, or "none" to not include it in programs:
[none] Simon Strandgaard (raspberrypi)

To estimate the required server capacity, the LODA miner
can send basic, anonymous usage statistics. Specifically,
a running miner instance would send the value 1 to the
API server once per hour. This data is used to determine
the total number of active miners. There are no IDs or other
data sent to the server. You can still mine without it.

Do you want to send this basic usage statisics? (y/N) y

Configure advanced settings? (y/N) 

===== Setup complete. Thanks for using LODA! =====

To run a Hello World example (Fibonacci numbers):
  loda eval A000045
To mine programs for OEIS sequences (single core):
  loda mine
To mine programs for OEIS sequences (multi core):
  loda mine -p
euclid@raspberrypi:~ $ source /home/euclid/.bashrc
euclid@raspberrypi:~ $ loda eval A40
2,3,5,7,11,13,17,19,23,29
euclid@raspberrypi:~ $
```


---

# Installing Rust language - used by LODA-RUST

I'm following [this install guide](https://www.rust-lang.org/tools/install).

```
euclid@raspberrypi:~ $ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
info: downloading installer

Welcome to Rust!

This will download and install the official compiler for the Rust
programming language, and its package manager, Cargo.

Rustup metadata and toolchains will be installed into the Rustup
home directory, located at:

  /home/euclid/.rustup

This can be modified with the RUSTUP_HOME environment variable.

The Cargo home directory is located at:

  /home/euclid/.cargo

This can be modified with the CARGO_HOME environment variable.

The cargo, rustc, rustup and other commands will be added to
Cargo's bin directory, located at:

  /home/euclid/.cargo/bin

This path will then be added to your PATH environment variable by
modifying the profile files located at:

  /home/euclid/.profile
  /home/euclid/.bashrc

You can uninstall at any time with rustup self uninstall and
these changes will be reverted.

Current installation options:


   default host triple: aarch64-unknown-linux-gnu
     default toolchain: stable (default)
               profile: default
  modify PATH variable: yes

1) Proceed with installation (default)
2) Customize installation
3) Cancel installation
>

info: profile set to 'default'
info: default host triple is aarch64-unknown-linux-gnu
info: syncing channel updates for 'stable-aarch64-unknown-linux-gnu'
info: latest update on 2022-11-03, rust version 1.65.0 (897e37553 2022-11-02)
info: downloading component 'cargo'
  6.2 MiB /   6.2 MiB (100 %)   4.2 MiB/s in  1s ETA:  0s
info: downloading component 'clippy'
info: downloading component 'rust-docs'
 18.9 MiB /  18.9 MiB (100 %)   3.8 MiB/s in  6s ETA:  0s
info: downloading component 'rust-std'
 40.0 MiB /  40.0 MiB (100 %)   3.4 MiB/s in 13s ETA:  0s
info: downloading component 'rustc'
 79.4 MiB /  79.4 MiB (100 %)   3.8 MiB/s in 27s ETA:  0s
info: downloading component 'rustfmt'
info: installing component 'cargo'
info: installing component 'clippy'
info: installing component 'rust-docs'
 18.9 MiB /  18.9 MiB (100 %)   1.3 MiB/s in 34s ETA:  0s    
info: installing component 'rust-std'
 40.0 MiB /  40.0 MiB (100 %)   3.8 MiB/s in 22s ETA:  0s
info: installing component 'rustc'
 79.4 MiB /  79.4 MiB (100 %)   4.4 MiB/s in 31s ETA:  0s
info: installing component 'rustfmt'
info: default toolchain set to 'stable-aarch64-unknown-linux-gnu'

  stable-aarch64-unknown-linux-gnu installed - rustc 1.65.0 (897e37553 2022-11-02)


Rust is installed now. Great!

To get started you may need to restart your current shell.
This would reload your PATH environment variable to include
Cargo's bin directory ($HOME/.cargo/bin).

To configure your current shell, run:
source "$HOME/.cargo/env"
euclid@raspberrypi:~ $ source "$HOME/.cargo/env"
euclid@raspberrypi:~ $ cargo --version
cargo 1.65.0 (4bc8f24d3 2022-10-20)
euclid@raspberrypi:~ $ 
```

# Installing Ruby language - used by LODA-RUST

I'm following [this install guide](https://www.ruby-lang.org/en/documentation/installation/).

```
euclid@raspberrypi:~ $ sudo apt-get update
Hit:1 http://security.debian.org/debian-security bullseye-security InRelease
Hit:2 http://deb.debian.org/debian bullseye InRelease           
Hit:3 http://deb.debian.org/debian bullseye-updates InRelease   
Hit:4 http://archive.raspberrypi.org/debian bullseye InRelease  
Reading package lists... Done                               
euclid@raspberrypi:~ $ sudo apt install ruby ruby-dev
Reading package lists... Done
Building dependency tree... Done
Reading state information... Done
The following package was automatically installed and is no longer required:
  libfuse2
Use 'sudo apt autoremove' to remove it.
The following additional packages will be installed:
  fonts-lato libruby2.7 rake ruby-minitest ruby-net-telnet ruby-power-assert ruby-rubygems ruby-test-unit ruby-xmlrpc ruby2.7
  rubygems-integration
Suggested packages:
  ri ruby-dev bundler
The following NEW packages will be installed:
  fonts-lato libruby2.7 rake ruby ruby-minitest ruby-net-telnet ruby-power-assert ruby-rubygems ruby-test-unit ruby-xmlrpc ruby2.7
  rubygems-integration
0 upgraded, 12 newly installed, 0 to remove and 97 not upgraded.
Need to get 8.052 kB of archives.
After this operation, 32,6 MB of additional disk space will be used.
Do you want to continue? [Y/n] 
Get:1 http://deb.debian.org/debian bullseye/main arm64 fonts-lato all 2.0-2.1 [2.696 kB]
Get:2 http://deb.debian.org/debian bullseye/main arm64 rubygems-integration all 1.18 [6.704 B]
Get:3 http://deb.debian.org/debian bullseye/main arm64 ruby2.7 arm64 2.7.4-1+deb11u1 [747 kB]
Get:4 http://deb.debian.org/debian bullseye/main arm64 ruby-rubygems all 3.2.5-2 [281 kB]
Get:5 http://deb.debian.org/debian bullseye/main arm64 ruby arm64 1:2.7+2 [11,7 kB]
Get:6 http://deb.debian.org/debian bullseye/main arm64 rake all 13.0.3-1 [84,7 kB]
Get:7 http://deb.debian.org/debian bullseye/main arm64 ruby-minitest all 5.13.0-1 [57,3 kB]
Get:8 http://deb.debian.org/debian bullseye/main arm64 ruby-net-telnet all 0.1.1-2 [12,5 kB]
Get:9 http://deb.debian.org/debian bullseye/main arm64 ruby-power-assert all 1.1.7-2 [11,5 kB]
Get:10 http://deb.debian.org/debian bullseye/main arm64 ruby-test-unit all 3.3.9-1 [86,1 kB]
Get:11 http://deb.debian.org/debian bullseye/main arm64 ruby-xmlrpc all 0.3.0-2 [23,7 kB]
Get:12 http://deb.debian.org/debian bullseye/main arm64 libruby2.7 arm64 2.7.4-1+deb11u1 [4.034 kB]
Fetched 8.052 kB in 2s (4.427 kB/s)      
Selecting previously unselected package fonts-lato.
(Reading database ... 96579 files and directories currently installed.)
Preparing to unpack .../00-fonts-lato_2.0-2.1_all.deb ...
Unpacking fonts-lato (2.0-2.1) ...
Selecting previously unselected package rubygems-integration.
Preparing to unpack .../01-rubygems-integration_1.18_all.deb ...
Unpacking rubygems-integration (1.18) ...
Selecting previously unselected package ruby2.7.
Preparing to unpack .../02-ruby2.7_2.7.4-1+deb11u1_arm64.deb ...
Unpacking ruby2.7 (2.7.4-1+deb11u1) ...
Selecting previously unselected package ruby-rubygems.
Preparing to unpack .../03-ruby-rubygems_3.2.5-2_all.deb ...
Unpacking ruby-rubygems (3.2.5-2) ...
Selecting previously unselected package ruby.
Preparing to unpack .../04-ruby_1%3a2.7+2_arm64.deb ...
Unpacking ruby (1:2.7+2) ...
Selecting previously unselected package rake.
Preparing to unpack .../05-rake_13.0.3-1_all.deb ...
Unpacking rake (13.0.3-1) ...
Selecting previously unselected package ruby-minitest.
Preparing to unpack .../06-ruby-minitest_5.13.0-1_all.deb ...
Unpacking ruby-minitest (5.13.0-1) ...
Selecting previously unselected package ruby-net-telnet.
Preparing to unpack .../07-ruby-net-telnet_0.1.1-2_all.deb ...
Unpacking ruby-net-telnet (0.1.1-2) ...
Selecting previously unselected package ruby-power-assert.
Preparing to unpack .../08-ruby-power-assert_1.1.7-2_all.deb ...
Unpacking ruby-power-assert (1.1.7-2) ...
Selecting previously unselected package ruby-test-unit.
Preparing to unpack .../09-ruby-test-unit_3.3.9-1_all.deb ...
Unpacking ruby-test-unit (3.3.9-1) ...
Selecting previously unselected package ruby-xmlrpc.
Preparing to unpack .../10-ruby-xmlrpc_0.3.0-2_all.deb ...
Unpacking ruby-xmlrpc (0.3.0-2) ...
Selecting previously unselected package libruby2.7:arm64.
Preparing to unpack .../11-libruby2.7_2.7.4-1+deb11u1_arm64.deb ...
Unpacking libruby2.7:arm64 (2.7.4-1+deb11u1) ...
Setting up fonts-lato (2.0-2.1) ...
Setting up ruby-power-assert (1.1.7-2) ...
Setting up rubygems-integration (1.18) ...
Setting up ruby-minitest (5.13.0-1) ...
Setting up ruby-test-unit (3.3.9-1) ...
Setting up ruby-net-telnet (0.1.1-2) ...
Setting up ruby-xmlrpc (0.3.0-2) ...
Setting up ruby2.7 (2.7.4-1+deb11u1) ...
Setting up ruby-rubygems (3.2.5-2) ...
Setting up ruby (1:2.7+2) ...
Setting up rake (13.0.3-1) ...
Setting up libruby2.7:arm64 (2.7.4-1+deb11u1) ...
Processing triggers for man-db (2.9.4-2) ...
Processing triggers for fontconfig (2.13.1-4.2) ...
Processing triggers for libc-bin (2.31-13+rpt2+rpi1+deb11u4) ...
euclid@raspberrypi:~ $ ruby --version
ruby 2.7.4p191 (2021-07-07 revision a21a3b7d23) [aarch64-linux-gnu]
euclid@raspberrypi:~/git/loda-rust/script $ sudo gem install bundler
Fetching bundler-2.3.26.gem
Successfully installed bundler-2.3.26
Parsing documentation for bundler-2.3.26
Installing ri documentation for bundler-2.3.26
Done installing documentation for bundler after 0 seconds
1 gem installed
euclid@raspberrypi:~/git/loda-rust/script $
```

# Installing OpenSSL devel - used by LODA-RUST

I'm following [this install guide](https://docs.rs/openssl/latest/openssl/).

```
euclid@raspberrypi:~ $ sudo apt-get install pkg-config libssl-dev
Reading package lists... Done
Building dependency tree... Done
Reading state information... Done
pkg-config is already the newest version (0.29.2-1).
The following package was automatically installed and is no longer required:
  libfuse2
Use 'sudo apt autoremove' to remove it.
Suggested packages:
  libssl-doc
The following NEW packages will be installed:
  libssl-dev
0 upgraded, 1 newly installed, 0 to remove and 97 not upgraded.
Need to get 1.699 kB of archives.
After this operation, 7.912 kB of additional disk space will be used.
Do you want to continue? [Y/n] 
Get:1 http://archive.raspberrypi.org/debian bullseye/main arm64 libssl-dev arm64 1.1.1n-0+deb11u3+rpt1 [1.699 kB]
Fetched 1.699 kB in 1s (2.786 kB/s)  
Selecting previously unselected package libssl-dev:arm64.
(Reading database ... 98595 files and directories currently installed.)
Preparing to unpack .../libssl-dev_1.1.1n-0+deb11u3+rpt1_arm64.deb ...
Unpacking libssl-dev:arm64 (1.1.1n-0+deb11u3+rpt1) ...
Setting up libssl-dev:arm64 (1.1.1n-0+deb11u3+rpt1) ...
euclid@raspberrypi:~ $
```


# Installing LODA-RUST

I'm following [this install guide](https://github.com/loda-lang/loda-rust/blob/develop/documents/install.md).

```
euclid@raspberrypi:~ $ mkdir git
euclid@raspberrypi:~ $ cd git
euclid@raspberrypi:~/git $ git clone https://github.com/loda-lang/loda-rust.git
Cloning into 'loda-rust'...
remote: Enumerating objects: 19945, done.
remote: Counting objects: 100% (4890/4890), done.
remote: Compressing objects: 100% (1113/1113), done.
remote: Total 19945 (delta 3938), reused 4697 (delta 3747), pack-reused 15055
Receiving objects: 100% (19945/19945), 4.90 MiB | 3.70 MiB/s, done.
Resolving deltas: 100% (13998/13998), done.
euclid@raspberrypi:~/git $ cd loda-rust/rust_project/
euclid@raspberrypi:~/git/loda-rust/rust_project $ pwd
/home/euclid/git/loda-rust/rust_project
euclid@raspberrypi:~/git/rust-loda/rust_project $ cargo build --release
   Compiling openssl-sys v0.9.77
   Compiling block-buffer v0.9.0
   Compiling cpufeatures v0.2.1
   Compiling num-integer v0.1.45
   ... snip ...
   Compiling loda-rust-web v0.1.0 (/home/euclid/git/loda-rust/rust_project/loda-rust-web)
   Compiling loda-rust-cli v2022.11.18 (/home/euclid/git/loda-rust/rust_project/loda-rust-cli)
    Finished release [optimized] target(s) in 15m 00s
euclid@raspberrypi:~/git/loda-rust/rust_project $ ./target/release/loda-rust 
loda-rust 0.0.1
Experimental tool

USAGE:
    loda-rust <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    dependencies    Print all direct/indirect dependencies of a program
    evaluate        Evaluate a program
    help            Print this message or the help of the given subcommand(s)
    install         Create the $HOME/.loda-rust directory
    mine            Run the miner daemon process. Press CTRL-C to stop it.
    pattern         Identify recurring patterns among similar programs.
    similar         Identify similar programs.
euclid@raspberrypi:~/git/loda-rust/rust_project $
```


Add the following $PATH to the bottom of $HOME/.bashrc
```
export PATH=$PATH:/home/euclid/git/loda-rust/rust_project/target/release
```

```
euclid@raspberrypi:~ $ loda-rust install
install success
euclid@raspberrypi:~ $ loda-rust eval A40
2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71
euclid@raspberrypi:~ $ 
```

# Getting the LODA-RUST miner working

Briefly run LODA-CPP to fetch OEIS stripped file and OEIS names file. When these have been fetched press CTRL-C to stop it.

```
euclid@raspberrypi:~ $ loda mine 
2022-11-27 15:54:14|INFO |Starting LODA v22.11.22. See https://loda-lang.org/
2022-11-27 15:54:14|INFO |Using LODA home directory "/home/euclid/loda/"
2022-11-27 15:54:14|INFO |Fetched https://raw.githubusercontent.com/loda-lang/loda-cpp/main/miners.default.json
2022-11-27 15:54:14|INFO |Creating OEIS index at "/home/euclid/loda/oeis/"
2022-11-27 15:54:20|INFO |Fetched http://api.loda-lang.org/miner/v1/oeis/stripped.gz
2022-11-27 15:54:24|INFO |Fetched http://api.loda-lang.org/miner/v1/oeis/names.gz
2022-11-27 15:54:24|INFO |Updating programs repository
^C2022-11-27 15:54:28|ERROR|Error executing command (exit code 2): git -C "/home/euclid/loda/programs/" pull origin main -q --ff-only
terminate called after throwing an instance of 'std::runtime_error'
  what():  Error executing command (exit code 2): git -C "/home/euclid/loda/programs/" pull origin main -q --ff-only
Aborted
euclid@raspberrypi:~ $
```


Check out the [loda-outlier-programs](https://github.com/neoneye/loda-outlier-programs) repo inside the $HOME/git dir.

```
euclid@raspberrypi:~/git $ git clone https://github.com/neoneye/loda-outlier-programs.git
Cloning into 'loda-outlier-programs'...
remote: Enumerating objects: 43233, done.
remote: Counting objects: 100% (4157/4157), done.
remote: Compressing objects: 100% (3671/3671), done.
remote: Total 43233 (delta 486), reused 3988 (delta 486), pack-reused 39076
Receiving objects: 100% (43233/43233), 12.29 MiB | 3.92 MiB/s, done.
Resolving deltas: 100% (6570/6570), done.
Updating files: 100% (44614/44614), done.
euclid@raspberrypi:~/git $ 
```

Install the Ruby dependencies that LODA-RUST needs for mining. 

```
euclid@raspberrypi:~/git/loda-rust/pwd $ script
/home/euclid/git/loda-rust/script
euclid@raspberrypi:~/git/loda-rust/script $ bundle install
Fetching gem metadata from https://rubygems.org/..
Using bundler 2.3.26
Following files may not be writable, so sudo is needed:
  /usr/local/bin
  /var/lib/gems/2.7.0
  /var/lib/gems/2.7.0/build_info
  /var/lib/gems/2.7.0/cache
  /var/lib/gems/2.7.0/doc
  /var/lib/gems/2.7.0/extensions
  /var/lib/gems/2.7.0/gems
  /var/lib/gems/2.7.0/plugins
  /var/lib/gems/2.7.0/specifications
Fetching parslet 2.0.0
Installing parslet 2.0.0
Fetching toml 0.3.0
Installing toml 0.3.0
Bundle complete! 1 Gemfile dependency, 3 gems now installed.
Use `bundle info [gemname]` to see where a bundled gem is installed.
euclid@raspberrypi:~/git/loda-rust/script $
```

Type in the nickname/name that you want to appear in the LODA programs that was mined on your device.

```
euclid@raspberrypi:~ $ nano .loda-rust/config.toml
loda_submitted_by = "Simon Strandgaard (raspberrypi)"
```

Launch the LODA-RUST miner

```
euclid@raspberrypi:~ $ loda-rust mine

██╗      ██████╗ ██████╗  █████╗       ██████╗ ██╗   ██╗███████╗████████╗
██║     ██╔═══██╗██╔══██╗██╔══██╗      ██╔══██╗██║   ██║██╔════╝╚══██╔══╝
██║     ██║   ██║██║  ██║███████║█████╗██████╔╝██║   ██║███████╗   ██║   
██║     ██║   ██║██║  ██║██╔══██║╚════╝██╔══██╗██║   ██║╚════██║   ██║   
███████╗╚██████╔╝██████╔╝██║  ██║      ██║  ██║╚██████╔╝███████║   ██║   
╚══════╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝      ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝
LODA-RUST version: 2022.11.18, build: RELEASE

metrics mode: NoMetricsServer
number of workers: 4
Press CTRL-C to stop the miner.


analytics_worker: child d38b18bd-9af4-4049-a080-c60b00f17bc6, received broadcast message: PerformSyncAndAnalytics
Successfully executed MinerSyncExecute. status: NoChange
BEFORE analytics - run_if_expired
Generating the "analytics" dir.

... snip ...

candidate: "20221127-151103-47186.asm"
miner_iterations: 3487
candidate: "20221127-151104-48297.asm"
miner_iterations: 977
miner_iterations: 468
miner_iterations: 65
MiningIsStoppingState: trigger start postmine
postmine_worker: child c8052478-60f2-40ed-9e8a-da8c642b0f43, received broadcast message: StartPostmineJob
BEFORE PostMine::run()
Number of pending programs: 41
Arrange programs by priority. high prio: 12, low prio: 29
    Finished Ran loda-cpp with pending programs, in 1 second
Looking up in the OEIS 'stripped' file
    Finished Lookups in the OEIS 'stripped' file, in 10 seconds
Minimizing programs
    Finished Minimized programs, in 1 second
Looking up in the OEIS 'names' file
    Finished Lookups in the OEIS 'names' file, in 2 seconds
Analyzing 63 program ids
████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 23/63
```

Now the LODA-RUST miner is running. Congrats.


