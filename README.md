# qctb

High-performance QC tools for bioinformatics analysis pipelines.

## Overview

`qctb` provides Rust implementations of QC summary reporting, offering 5-6x performance improvements while maintaining full compatibility with existing workflows.

## Features

- **High Performance**: 5-6x faster than R versions for QC summary
- **Memory Efficient**: Optimized memory usage with streaming parsers
- **Type Safe**: Rust's type system prevents runtime errors
- **Parallel Processing**: Multi-core utilization with Rayon
- **Easy Integration**: Drop-in replacement for R scripts

## Usage

### Standard Mode (RRBS/WGBS)

Generate QC summary reports with BS-seq alignment statistics.

```bash
qctb --config config.yaml --output qc_summary.xlsx
```

### RNA-seq Mode

Generate QC summary reports with STAR alignment statistics.

```bash
qctb --config config.yaml --output qc_summary.xlsx --rnaseq
```

### Output Format Options

Specify output format (Excel or TSV):

```bash
# Excel format (default)
qctb --config config.yaml --output qc_summary.xlsx

# TSV format
qctb --config config.yaml --output qc_summary.tsv --format tsv
```

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/xdxtools/qctb.git
cd qctb

# Build release version
cargo build --release

# Binary location
./target/release/qctb
```

### As xdxtools Submodule

```bash
# In xdxtools directory
git submodule add https://github.com/xdxtools/qctb.git qctb
cd qctb
cargo build --release
```

## Performance

| Task | R Version | Rust Version | Speedup |
|------|-----------|--------------|---------|
| QC Summary (10 samples) | 30-60s | 5-10s | **5-6x** |

## Documentation

- [Requirements](docs/requirements.md) - Technical requirements
- [Design](docs/design.md) - Architecture and design decisions
- [API Reference](docs/api.md) - CLI API documentation
- [Active Context](docs/active_context.md) - Current development status

## Development

### Prerequisites

- Rust 1.70+
- Cargo

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Release with LTO
cargo build --profile release.lto
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

## License

MIT License - see LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
