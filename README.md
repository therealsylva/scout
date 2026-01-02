# Scout

Blazing fast extension counter and file analyzer written in Rust.

## Features

- **High Performance**: Parallel processing using Rayon for lightning-fast analysis
- **Multiple Sorting Options**: Sort by count, size, date, or extension
- **Flexible Output**: Human-readable table output or JSON for integration
- **Detailed Analysis**: View file counts, sizes, and timestamps
- **Smart Traversal**: Respects `.gitignore` and other ignore files
- **Customizable**: Filter by specific extensions or analyze all files

## Installation

### Cargo

```bash
cargo install scout
```

### From Source

```bash
git clone https://github.com/therealsylva/scout.git
cd scout
cargo install --path .
```

## Usage

Basic usage - analyze current directory:

```bash
scout
```

Analyze a specific directory:

```bash
scout --target /path/to/directory
```

Filter by extension:

```bash
scout --extension rs
```

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--extension` | `-e` | Filter by specific file extension |
| `--target` | `-t` | Target directory to analyze (default: current directory) |
| `--sort-by` | `-s` | Sort results by `count`, `size`, `date`, or `extension` |
| `--json` | `-j` | Output results in JSON format |
| `--detailed` | `-d` | Show detailed output with timestamps |

### Examples

Count Rust files sorted by size:

```bash
scout --extension rs --sort-by size
```

Detailed analysis of all files:

```bash
scout --detailed
```

JSON output for automation:

```bash
scout --json > results.json
```

Find most recent Python files:

```bash
scout --extension py --sort-by date --detailed
```

## Performance

Scout is optimized for speed with:
- **Parallel Processing**: Uses Rayon for concurrent file processing
- **Optimized Build**: Enabled LTO and aggressive optimizations in release mode
- **Efficient Traversal**: Built on the high-performance `ignore` crate (used by ripgrep)

## Building from Source

```bash
# Debug build
cargo build

# Release build (recommended)
cargo build --release

# Run tests
cargo test
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License.
