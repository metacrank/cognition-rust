# Introduction
Cognition was a programming language designed as a universal interpreter or _metalanguage_. However,
the previous C implementation lacked the security needed in order to build on top of it. Therefore,
there is a need for a memory safe implementation of cognition in order for it to be a suitable base-layer
for programs written in cognition. This cognition has a similar architecture, but it different in implementation.
We still call it a cognition because it has all the characteristics that a cognition should have: namely,
the ability to interpret arbitrary text input with an arbitrarily complicated, dynamically re-programmable rule,
and Turing completeness.

## Installation
Here we provide the installation instructions for supported operating systems and distributions.
### NixOS
```sh
nix install
```
### GNU Guix
``` sh
guix package -f cognition.scm
```
### Installing from source
To install Cognition from source, use the cargo commands:

```sh
cargo build --release
cargo install
```
Alternatively, to run the Cognition interpreter without installing it,

```sh
cargo run
```

### Installing libraries
It is recommended to keep a local copy of the coglib and associated fllibs in a ```~/.cognition``` directory.
Until the build system is modified to automate the creation and population of this directory, this step must be performed manually.

```sh
cd cognition-rust
mkdir ~/.cognition
cp -r coglib ~/.cognition
mkdir ~/.cognition/fllib
for d in fllib/*/; do; (cd $d && cargo build --release && cp target/release/*.so ~/.cognition/fllib); done
```

## Usage
Since most Cognition applications will rely on the std/bootstrap.cog file or other parts of the standard library,
it is recommended to set the ```COGLIB_DIR``` environment variable before running any of the examples below
(for instance, ```COGLIB_DIR=/home/user/.cognition/coglib``` or ```COGLIB_DIR=/home/user/src/cognition-rust/coglib```).
The ```crank``` executable will search this directory if a source file is not found in the current directory.

Alternatively, full file paths can be supplied for each source file, or the following examples can be run in the
```coglib``` subdirectory.

### The REPL
A repl with just the standard library loaded can be accessed with the following command:

```sh
crank -s 2 std/bootstrap.cog std/repl.cog
```

To get a fully fledged Cognition repl, first install all foreign language libraries (fllibs) as described in 'Installing libraries'.

Now you can use this command:

```sh
crank -s 4 std/bootstrap.cog std.cog examples/common-fllibs.cog utils/repl.cog
```

To open a new terminal emulator with the cognition repl, consider modifying the shell script ```cognition-repl.sh``` in ```./scripts``` to use your favourite emulator.
This will require you to set the ```COGLIB_DIR``` envvar globally for your user.

<div align="center"><img src="assets/images/logos/cog.png" width="50" height="50"></div>
