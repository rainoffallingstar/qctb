# qctb Active Context

**Last Updated**: 2026-02-22
**Current Version**: qctb 0.1.0 (renamed from xdxtools-bio)

## Implementation Status

### Completed ✅
- [x] Project renamed from xdxtools-bio to qctb
- [x] CLI structure simplified (removed subcommands)
- [x] repair_unpaired_reads module removed
- [x] Documentation updated (README, API, requirements, design)
- [x] Cargo.toml metadata updated
- [x] QC summary config parser (YAML)
- [x] seqkit parser (reads, bases, Q20/Q30 stats)
- [x] trim_galore parser (adapter trimming stats)
- [x] QC data aggregator (process samples)
- [x] STAR aligner parser (Log.final.out)
- [x] RNA-seq mode with --rnaseq flag
- [x] bismark parser (BS-seq alignment stats)
- [x] qualimap parser (mapping quality, duplication)
- [x] Excel (.xlsx) output with rust_xlsxwriter
- [x] TSV/Excel dual output support via --format flag
- [x] Unit tests (8/8 passing)
- [x] Release build script (build-release.sh)

## Current Focus

**v0.1.0 Release (2026-02-22)**: ✅ Project renamed and refactored
- ✅ Binary name: `qctb`
- ✅ Simplified CLI: `qctb [OPTIONS]` (no subcommands)
- ✅ Removed repair-unpaired functionality
- ✅ All documentation updated
- ✅ Focus on QC summary reporting only

**Working Commands**:
```bash
# Standard QC summary (BS-seq/WGBS mode) - Excel output
qctb --config config.yaml --output qc_summary.xlsx

# Standard QC summary - TSV output
qctb --config config.yaml --output qc_summary.txt --format tsv

# RNA-seq mode (with STAR alignment stats) - Excel output
qctb --config config.yaml --output qc_summary.xlsx --rnaseq

# RNA-seq mode - TSV output
qctb --config config.yaml --output qc_summary.txt --format tsv --rnaseq
```

**Output Formats**:
- **Excel (.xlsx)**: Professional formatting with colored headers, borders, auto-sized columns
- **TSV**: Tab-separated values, compatible with Excel and other tools
- Standard mode: 25 columns (seqkit + trim + bismark + qualimap)
- RNA-seq mode: 22 columns (seqkit + trim + STAR)

## Technical Decisions Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-02-22 | Rename to qctb | Shorter, more focused name |
| 2026-02-22 | Remove subcommands | Simpler CLI, single purpose |
| 2026-02-22 | Remove repair-unpaired | Focus on QC summary reporting |
| 2026-01-14 | Use `--rnaseq` flag | Simpler than separate command |
| 2026-01-14 | STAR parser for RNA-seq | Parses Log.final.out |
| 2026-01-14 | Separate QCSummary structs | Type-safe separation |
| 2026-01-14 | TSV/Excel output support | Multiple format options |

## Known Issues

- None yet

## Dependencies

- Rust 1.70+
- Cargo
- All dependencies in Cargo.toml locked

## Current Working Features

### Standard Mode (BS-seq/WGBS)
- **Function**: Aggregate QC metrics from seqkit, trim galore, bismark, qualimap
- **Input**: YAML config + QC files from multiple tools
- **Output**: Excel or TSV file with 25 columns
- **Supported metrics**:
  - seqkit: reads, bases, Q20/Q30, lengths (R1/R2, raw/clean)
  - trim_galore: adapter trimming, quality trimming stats
  - bismark: alignment ratio, total/aligned reads
  - qualimap: mapping quality, duplication rate

### RNA-seq Mode
- **Function**: Aggregate QC metrics from seqkit, trim galore, STAR
- **Input**: YAML config + STAR Log.final.out
- **Output**: Excel or TSV file with 22 columns
- **Supported metrics**:
  - seqkit: reads, bases, Q20/Q30, lengths (R1/R2, raw/clean)
  - trim_galore: adapter trimming, quality trimming stats
  - STAR: mapping ratio, total reads, uniquely mapped reads
- **Usage**: Add `--rnaseq` flag

### Build System
- **Script**: `build-release.sh`
- **Profile**: Release with LTO=fat, codegen-units=1, strip=true
- **Binary Size**: ~3.4MB (includes all parsers and Excel support)

## Test Commands

```bash
# Run unit tests
cargo test

# Run specific test
cargo test test_parse_star_log
cargo test test_parse_seqkit

# Build release
./build-release.sh

# Test QC summary (standard mode)
./target/release/qctb \
  --config /path/to/config.yaml \
  --output /path/to/qc_summary.xlsx

# Test QC summary (RNA-seq mode)
./target/release/qctb \
  --config /path/to/config.yaml \
  --output /path/to/qc_summary.xlsx \
  --rnaseq

# Test TSV output
./target/release/qctb \
  --config /path/to/config.yaml \
  --output /path/to/qc_summary.tsv \
  --format tsv
```
