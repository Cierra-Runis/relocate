## Setup

```sh
rustup override set nightly
```

## Graph

To generate a module dependency graph, run:

```sh
cargo install cargo-modules
cargo modules dependencies --package relocate-midi --layout dot --no-externs --no-fns --no-uses > ./target/mods-owns.dot
cargo modules dependencies --package relocate-midi --layout circo --no-externs --no-fns --no-owns > ./target/mods-uses.dot
cargo modules dependencies --package relocate-midi --layout sfdp --no-externs --no-fns > ./target/mods.dot
```

Then use [GraphvizOnline](https://dreampuf.github.io/GraphvizOnline?engine=neato) to visualize the file.

## Specification

- [Standard MIDI Files 1.0](https://drive.google.com/file/d/1t4jcCCKoi5HMi7YJ6skvZfKcefLhhOgU/view)
- [MIDI Clip File Specification – SMF MIDI 2.0](https://midi.org/midi-clip-file-specification-smf-midi-2-0)
