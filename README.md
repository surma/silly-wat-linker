# Silly WAT Linker

SWL is a tool that makes writing [WebAssembly Text files][wat spec] easier.

It is future-proof and simple because it doesn’t actually understand WAT. Instead, it is a simplistic [S-Expression] parser and uses surface-level pattern matching to implement its features. This way, future iterations of WAT with new syntax are unlikely to break this tool.

WAT is the WebAssembly Text format and is [defined][wat spec] in the [WebAssembly spec].

## Usage

SWL can be installed from Cargo:

```
$ cargo install silly-wat-linker
```

All work is done on the textual representation. SWL can invoke [wabt’s `wat2wasm`][wabt] for you to produce a binary file instead:

```
$ silly-wat-linker ./main.wat      # Emits .wat test file
$ silly-wat-linker -c ./main.wat      # Emits .wasm binary
```

## Features

SWL has a handful of features to make your life easier when hand-writing WAT files. Most features are enabled by default, but you can explicitly select which features to enable using the `--features` flag.

```
# This will only run the `size_adjust` and `sort` feature.
$ silly-wat-linker --features size_adjust,sort ./my-file.wat
```

### File Importer (`import`)

Adds support for importing another .wat file into the current one.

```wat
(module
	(import "other_file.wat" (file))
	;; ...
)
```

### Size Adjuster (`size_adjust`)

Automatically adjust the size of `memory` directives to be big enough to hold all active `data` segments. (This feature is also supposed to do the same for `tables` and `elem` segments, but this hasn’t been implemented yet.)

### Start Merger (`start_merge`)

If there are multiple `(start)` directives (which can easily happen in a multi-file project), SWL will create a new, singular start function that calls all the other start functions.

### Sorter (`sort`)

Sorts all top-level module segments so that imports come first. This feature mostly exists because `wat2wasm` requires imports to come first.

---

License Apache-2.0

[wat spec]: https://webassembly.github.io/spec/core/text/index.html
[webassembly spec]: https://webassembly.github.io/spec/core/
[wabt]: https://github.com/WebAssembly/wabt
[s-expression]: https://en.wikipedia.org/wiki/S-expression
