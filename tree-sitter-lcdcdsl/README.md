# tree-sitter-lcdcdsl

This sub-repo contains the parser for LangCities DC's Domain Specific Language. It uses [tree-sitter](https://tree-sitter.github.io/tree-sitter/index.html), a parser generator tool with pretty neat performance numbers. Bindings for tree-sitter's officially-supported programming languages can be found pre-configured in this directory, but I will mainly be using the Rust and JavaScript bindings.

Instructions are pulled mainly from [here](https://tree-sitter.github.io/tree-sitter/index.html).

## Requirements

`tree-sitter` requires the `tree-sitter-cli` in order to build it. Grab it from crates.io:

```bash
cargo install tree-sitter-cli
```

Ensure that `$HOME/.rust/cargo/bin` is in `$PATH` or the equivalent or you won't be able to use `tree-sitter`.

## Build

If you haven't already, `cd` into the sub-repo

```bash
cd tree-sitter-lcdcdsl
```

Generate bindings for support programming languages.

```bash
tree-sitter generate
```

That's it!

## Test

Tests are stored under `tree-sitter-lcdcdsl/test/corpus/statements.txt`. Refer to the official tree-sitter documentation to discover how to add more tests.

```bash
tree-sitter test
```

Benchmarks using `hyperfine` on a machine equipped with a U9 275HX:

```bash
hyperfine --warmup 3 --min-runs 5 --shell=none 'tree-sitter test'
# Benchmark 1: tree-sitter test
#   Time (mean ± σ):       1.7 ms ±   0.1 ms    [User: 0.9 ms, System: 0.8 ms]
#   Range (min … max):     1.6 ms …   2.3 ms    1505 runs
```
