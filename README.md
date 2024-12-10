# Introduction
Cognition was a programming language designed as a universal interpreter or _metalanguage_. However,
the previous C implementation lacked the security needed in order to build on top of it. Therefore,
there is a need for a memory safe implementation of cognition in order for it to be a suitable base-layer
for programs written in cognition. This cognition has a similar architecture, but it different in implementation.
We still call it a cognition because it has all the characteristics that a cognition should have: namely,
the ability to interpret arbitrary text input with an arbitrarily complicated, dynamically re-programmable rule,
and Turing completeness.

## Install
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
Alternatively, to run cognition without installing,

```sh
cargo run
```

## Usage

### The REPL
To get a fully fledged cognition repl, first run 'cargo build --release' in each of the ```./fllib``` subdirectories.

Then set the ```COGLIB_DIR``` envvar (for instance, ```COGLIB_DIR=/home/user/src/cognition-rust/coglib```).

Now you can use the following command:

```sh
crank -s 4 stdbootstrap.cog stdquote.cog common-fllibs.cog repl.cog
```

Alternatively, you can specify the path to each cog file or run the command in the coglib subdirectory.

To open a new terminal emulator with the cognition repl, consider modifying the shell script ```cognition-repl.sh``` in ```./scripts``` to use your favourite emulator.
This will require you to set the ```COGLIB_DIR``` envvar globally for your user.

<div align="center"><img src="assets/images/logos/cog.png" width="50" height="50"></div>
