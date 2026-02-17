# bpwd

A better pwd with clipboard support and targeting.

## Installation

### Homebrew (macOS/Linux)

```bash
brew install CodeTease/tap/bpwd
```

### Cargo

```bash
cargo install bpwd
```

### Installer Script

In Linux:
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/CodeTease/bpwd/releases/latest/download/bpwd-installer.sh | sh
```

In Windows (PowerShell)
```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/CodeTease/bpwd/releases/latest/download/bpwd-installer.ps1 | iex"
```

### Release

You can also download the binary from the [Release page](https://github.com/CodeTease/bpwd/releases).

## Usage

```bash
bwd [target] [-c] [-s] [-r]
```

- `target`: Optional path to resolve relative to current directory. Use `--` to separate flags from arguments (e.g., `bwd -- -my-dir`).
- `-c`: Copy the result to clipboard.
- `-s`: Use forward slashes (`/`) instead of backslashes (`\`).
- `-r`: Print path relative to project root (searches for `.git` or `.bwd-root`).
