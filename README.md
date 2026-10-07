# Language Jutsu

A Rust CLI project for preparing foreign-language recordings for repeated listening practice. Initial priorities are Arabic (Shami and Masri) and Japanese, with Spanish also included.

The skeleton implements CLI parsing, language/dialect settings, strict JSON formula loading, and configuration checks. Audio decoding, pause-based segmentation, sequence expansion, time stretching, and audio export remain to be implemented. A render request currently exits with an explicit error and writes no output.

## Run the skeleton

```sh
cargo run -- --help
cargo run -- recording.wav --language ar --dialect shami \
  --formula formulas/default.json --output study.mp3 --check
```

`--check` verifies that the input is a regular file and that the formula is valid. It does not inspect or validate the audio contents. It reports the supplied language and every formula setting without creating an output file.

Without `--formula`, the CLI reads `formulas/default.json` relative to its executable's directory. For development, use the explicit repository path shown above. A distributed binary must be accompanied by that JSON file; automatic packaging is not part of the skeleton. Missing or invalid formulas are errors. Recipe values are never compiled into the program, and the processing entry point requires a parsed formula.

The [default formula](formulas/default.json) is editable. It defines speed order, sequencing mode, joining group size, repetitions, and inter-play gaps. All fields are required; custom files replace the formula completely. Ordered and duplicate speed entries are preserved.

## Design and research

- [First-pass CLI design](design/README.md)
- [Formula semantics and worked sequences](design/formula.md)
- [CLI and processing model](design/cli-and-processing.md)
- [Listening research](research/README.md)

## Development checks

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```
