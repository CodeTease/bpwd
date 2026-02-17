# bpwd

A better pwd with clipboard support and targeting.

## Installation

### Homebrew

```bash
brew install CodeTease/tap/bpwd
```

### Cargo

```bash
cargo install bpwd
```

### Release

You can also download the binary from the [Release page](https://github.com/CodeTease/bpwd/releases).

## Usage

```bash
bwd [target] [-c] [-s]
```

- `target`: Optional path to resolve relative to current directory.
- `-c`: Copy the result to clipboard.
- `-s`: Use forward slashes (`/`) instead of backslashes (`\`).
