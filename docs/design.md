# qctb Architecture Design

## Architecture Overview

```
┌─────────────┐
│   CLI       │  clap-based command parsing
│  (main.rs)  │
└──────┬──────┘
       │
       └─> qc_summary module
            ├─> config: Load and parse YAML
            ├─> parsers: Parse various QC file formats
            │    ├─> seqkit: CSV parsing
            │    ├─> trim_galore: Regex-based text parsing
            │    ├─> bismark: Key-value extraction
            │    ├─> qualimap: XML/Text parsing
            │    └─> rnaseq: RNA-seq specific parsers
            ├─> aggregator: Data aggregation
            └─> excel: Excel report generation
```

## Key Design Decisions

### 1. Single Binary with Direct Options
**Decision**: Use `qctb [OPTIONS]` instead of subcommands
**Rationale**: Simpler CLI, focused on single use case (QC summary), better UX

### 2. Flag-based Mode Selection
**Decision**: Use `--rnaseq` flag instead of separate command
**Rationale**: Share core logic, reduce duplication, easier maintenance

### 3. Optional Fields for Mode-Specific Data
**Decision**: Use `Option<T>` for mode-specific fields (bismark_stats, star_stats)
**Rationale**: Type-safe, clear semantics, no wasted memory

## Data Flow

### QC Summary Flow
```
config.yaml → Config Parser → QCConfig struct
     ↓
Sample List
     ↓
For each sample:
  seqkit.txt → SeqkitStats
  trim.txt → TrimStats
  [mode-specific] → ModeStats
     ↓
Aggregate into QCSummary/QCSummaryRNA struct
     ↓
All summaries → Excel/TSV Writer → output.{xlsx,tsv}
```

## Error Handling Strategy

### Input Validation
- Check file existence before parsing
- Validate YAML structure
- Verify required fields present

### Parsing Errors
- Use `anyhow::Result<T>` for error propagation
- Provide context with `.context()` calls
- Include file path and line number in errors

### Graceful Degradation
- Log warnings for missing optional files
- Skip samples with critical errors
- Return clear exit codes

## Performance Optimization

### Memory Management
- Use streaming parsers for large files
- Avoid loading entire dataset into memory
- Drop temporary structs early

## Output Formats

### Excel Output
- Professional formatting with colored headers
- Auto-sized columns
- Border styling

### TSV Output
- Tab-separated values
- Compatible with Excel and other tools
- Standard and RNA-seq modes have different column counts
