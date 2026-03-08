# qctb CLI API Reference

## Overview

qctb provides bioinformatics QC tools implemented in Rust for high performance.

## Command

Generate QC summary report from multiple tool outputs.

```bash
qctb [OPTIONS]
```

**Options**:
- `--config <FILE>` - YAML configuration file (required)
- `--output <FILE>` - Output file path (required)
- `--format <fmt>` - Output format: xlsx or tsv (default: xlsx)
- `--rnaseq` - Enable RNA-seq mode (use RNA-seq specific metrics)

**Standard Mode**: Parses seqkit, trim_galore, bismark, qualimap outputs
**RNA-seq Mode**: Parses seqkit, trim_galore, STAR outputs

**Exit Codes**:
- `0` - Success
- `1` - Error (check stderr for details)

**Examples**:
```bash
# Standard QC summary (BS-seq)
qctb \
  --config config/config.yaml \
  --output qc_summary.xlsx

# Standard QC summary (TSV format)
qctb \
  --config config/config.yaml \
  --output qc_summary.tsv \
  --format tsv

# RNA-seq QC summary
qctb \
  --config config/config.yaml \
  --output qc_summary_rnaseq.xlsx \
  --rnaseq
```

## Global Options

- `-h, --help` - Print help information
- `-V, --version` - Print version information

## Configuration File Format

The YAML config file must contain:

```yaml
SIDs:
  - sample1
  - sample2
qcDir: "/path/to/qc/output"
trimDir: "/path/to/trim/output"
bsmapDir: "/path/to/bsmap"
outDir_mCall: "/path/to/mcall"
# Optional fields
graft: "human"  # or other species
qcdir_before: "/path/to/before/qc"
qcdir_after: "/path/to/after/qc"
```

## Output Formats

### Excel Output
- `.xlsx` format with professional formatting
- Colored headers and borders
- Auto-sized columns
- Single summary sheet with all samples

### TSV Output
- Tab-separated values format
- Compatible with Excel and other tools
- Standard mode: 25 columns
- RNA-seq mode: 22 columns
