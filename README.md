# bpwd

A better pwd with clipboard support and targeting.

## Installation

### Homebrew (macOS/Linux)

```bash
brew install CodeTease/tap/bwd
```

### Scoop (Windows)

```bash
scoop bucket add codetease https://github.com/CodeTease/scoop-bucket
scoop install bwd
```

### Cargo

```bash
cargo install bpwd
```

### Release

You can also download the binary from the [Release page](https://github.com/CodeTease/bpwd/releases).

For more download methods, see [INSTALLATION.md](INSTALLATION.md).

## Usage

```bash
bwd [target] [-c] [-s] [-j] [-r]
```

- `target`: Optional path to resolve relative to current directory. Use `--` to separate flags from arguments (e.g., `bwd -- -my-dir`).
- `-c`: Copy the result to clipboard.
- `-s`: Shorten path (replace home directory with `$HOME` or `%USERPROFILE%`).
- `-j`: Output path information as JSON.
- `-r`: Print path relative to project root (searches for `.git` or `.bwd-root`).

### Examples

**Standard Output (Absolute Path)**
```bash
$ bwd
/home/codetease/projects/bpwd
```

**Shortened Path (`-s`)**
```bash
$ bwd -s
$HOME/projects/bpwd
```

**JSON Output (`-j`)**
```bash
$ bwd -j
{"path":"/home/codetease/projects/bpwd","short":"$HOME/projects/bpwd","root":"."}
```

**Priority Logic**
1. **JSON (`-j`)**: Always outputs the JSON object.
2. **Short (`-s`)**: If JSON is not requested, outputs the shortened path.
3. **Default**: Outputs the absolute path.
