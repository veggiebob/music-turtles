# Music Turtles

Lindenmayer-systems (L-systems) are formal grammars that generate strings through parallel rewriting, where user-defined production rules are applied simultaneously to all symbols at each iteration. L-systems are usually used for procedural graphics and plant modeling, but here they are repurposed for music composition through the development of a domain-specific language and tool that maps strings to musical instructions which are rendered using Musical Instrument Digital Interface (MIDI). The language enables modular composition through systematic replacement and non-destructive transformations such as repetition, transposition, and tempo modification of musical fragments. Procedural generation is also supported through probabilistic rule selection, enabling both fixed-length and recursively expanding compositions. Beyond generative outputs, the system's capacity to faithfully encode classical works is demonstrated, including Bach's *Prelude in C Major*. Avenues for future development include extended instruction sets and improved tonal awareness. This tool offers a novel perspective on algorithmic composition through the lens of formal grammars.

## Features

- **Domain-specific language** for describing musical L-systems.
- **Non-destructive transformations**: repeat sections with `[xN]`, transpose with `[TN]`, or compress/expand time with `[>>N]`.
- **Probabilistic rule selection** for procedural generation.
- **Playback engines** for local audio (sine wave synthesis via `rodio`) or for external gear using MIDI (`midir`).
- **Unit tests** covering core components such as time handling, composition utilities, and scheduler logic.

## Repository layout

```
├── data/           # example grammars and compositions
├── frontend/       # incomplete prototype UI (ignored)
├── music-turtles/  # backend Rust crate
└── Cargo.toml      # workspace definition
```

The `data/` directory contains ready-made `.mtx` examples. For instance, `stress-test1.mtx` generates a simple beat:

```text
start S

S = M M M M

M = { Boom | Clap }

Boom = :3c :_  :3c :_
Clap = :_  :3g :_  :3g
```

Transformation syntax can be seen in `funky_bach.mtx`:

```text
19 = [T-12][1]                       # transpose phrase 1 down an octave
1  = [x2][{:c<8> | :_ :e<7> | :_<2> [x2][C6/4] }]
```

## Getting started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- ALSA development libraries (on Linux) for `rodio`/`midir`

### Build

```bash
cargo build
```

### Run

Running the default binary loads the `funky_bach.mtx` grammar and plays the result via MIDI:

```bash
cargo run
```

> **Note:** The engine only sends raw MIDI messages and must be configured manually to route them to an output device. To hear sound, connect the output to a DAW or hardware synthesizer. A free, open-source option is [LMMS](https://lmms.io/), which can receive the MIDI stream and render audio.

### Tests

```bash
cargo test
```

> **Note:** tests and playback require the ALSA system library. If `cargo test` fails with an `alsa` error, install the `libasound2-dev` package or provide the appropriate `PKG_CONFIG_PATH`.

## References

- [Rodio](https://github.com/RustAudio/rodio) – audio playback
- [midir](https://github.com/Boddlnagg/midir) – MIDI I/O

## License

This project is licensed under the MIT License.
