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

