use super::seqkit::SeqkitStats;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct FqcRow {
    pub num_seqs: u64,
    pub sum_len: u64,
    pub q20: f64,
    pub q30: f64,
    pub min_len: u32,
    pub avg_len: f64,
    pub max_len: u32,
}

/// Parse the >>Seqkit Statistics section from a fastqc_data.txt file produced by fqc.
/// The section uses a tabular format: a header row (#file\tformat\t...) followed by
/// a single data row, with >>END_MODULE appended to the end of the data row.
pub fn parse_fqc_data(file_path: &str) -> Result<FqcRow> {
    let file = File::open(file_path)
        .with_context(|| format!("Failed to open fqc data file: {}", file_path))?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .collect::<std::io::Result<Vec<String>>>()
        .with_context(|| format!("Failed to read fqc data file: {}", file_path))?;

    let mut in_section = false;
    let mut col_idx: HashMap<String, usize> = HashMap::new();
    let mut kv: HashMap<String, f64> = HashMap::new();

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with(">>Seqkit Statistics") {
            in_section = true;
            continue;
        }
        if !in_section {
            continue;
        }
        // Strip >>END_MODULE suffix (appears appended to the data row)
        let clean = trimmed.trim_end_matches(">>END_MODULE").trim_end();
        if clean.starts_with(">>") {
            break;
        } // Standalone >>END_MODULE or next section
        if clean.starts_with('#') {
            // Header row: build column-name → index mapping
            let headers: Vec<&str> = clean.trim_start_matches('#').split('\t').collect();
            for (i, h) in headers.iter().enumerate() {
                col_idx.insert(h.trim().to_string(), i);
            }
            continue;
        }
        // Data row: extract values by column index
        if !col_idx.is_empty() {
            let values: Vec<&str> = clean.split('\t').collect();
            for (col_name, &idx) in &col_idx {
                if idx < values.len() {
                    if let Ok(val) = values[idx].trim().parse::<f64>() {
                        kv.insert(col_name.clone(), val);
                    }
                }
            }
        }
    }

    let get = |key: &str| -> Result<f64> {
        kv.get(key).copied().with_context(|| {
            format!(
                "Key '{}' not found in Seqkit Statistics of {}",
                key, file_path
            )
        })
    };

    Ok(FqcRow {
        num_seqs: get("num_seqs")? as u64,
        sum_len: get("sum_len")? as u64,
        q20: get("Q20(%)")?,
        q30: get("Q30(%)")?,
        min_len: get("min_len")? as u32,
        avg_len: get("avg_len")?,
        max_len: get("max_len")? as u32,
    })
}

/// Read 4 fqc fastqc_data.txt files and assemble into SeqkitStats
pub fn parse_seqkit_from_fqc(
    raw_r1: &str,
    raw_r2: &str,
    clean_r1: &str,
    clean_r2: &str,
) -> Result<SeqkitStats> {
    let r1 = parse_fqc_data(raw_r1)?;
    let r2 = parse_fqc_data(raw_r2)?;
    let c1 = parse_fqc_data(clean_r1)?;
    let c2 = parse_fqc_data(clean_r2)?;

    let reads_raw = r1.num_seqs + r2.num_seqs;
    let bases_raw = r1.sum_len + r2.sum_len;
    let reads_clean = c1.num_seqs + c2.num_seqs;
    let bases_clean = c1.sum_len + c2.sum_len;
    let clean_data_ratio = if bases_raw > 0 {
        bases_clean as f64 / bases_raw as f64
    } else {
        0.0
    };

    Ok(SeqkitStats {
        reads_raw_r1: r1.num_seqs,
        bases_raw_r1: r1.sum_len,
        q20_raw_r1: r1.q20,
        q30_raw_r1: r1.q30,
        min_len_raw_r1: r1.min_len,
        avg_len_raw_r1: r1.avg_len,
        max_len_raw_r1: r1.max_len,

        reads_raw_r2: r2.num_seqs,
        bases_raw_r2: r2.sum_len,
        q20_raw_r2: r2.q20,
        q30_raw_r2: r2.q30,
        min_len_raw_r2: r2.min_len,
        avg_len_raw_r2: r2.avg_len,
        max_len_raw_r2: r2.max_len,

        reads_clean_r1: c1.num_seqs,
        bases_clean_r1: c1.sum_len,
        q20_clean_r1: c1.q20,
        q30_clean_r1: c1.q30,
        min_len_clean_r1: c1.min_len,
        avg_len_clean_r1: c1.avg_len,
        max_len_clean_r1: c1.max_len,

        reads_clean_r2: c2.num_seqs,
        bases_clean_r2: c2.sum_len,
        q20_clean_r2: c2.q20,
        q30_clean_r2: c2.q30,
        min_len_clean_r2: c2.min_len,
        avg_len_clean_r2: c2.avg_len,
        max_len_clean_r2: c2.max_len,

        reads_raw,
        bases_raw,
        reads_clean,
        bases_clean,
        clean_data_ratio,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_fqc_data() -> Result<()> {
        // Mimics the actual fqc tabular output format (fastqc-rs template)
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, ">>Basic Statistics\tpass")?;
        writeln!(temp_file, "#Measure\tValue")?;
        writeln!(temp_file, "Filename\ttest.fastq.gz")?;
        writeln!(temp_file, ">>END_MODULE")?;
        writeln!(temp_file, ">>Seqkit Statistics\tpass")?;
        writeln!(temp_file, "#file\tformat\ttype\tnum_seqs\tsum_len\tmin_len\tavg_len\tmax_len\tQ1\tQ2\tQ3\tsum_gap\tN50\tN50_num\tQ20(%)\tQ30(%)\tAvgQual\tGC(%)\tsum_n")?;
        writeln!(temp_file, "test.fastq.gz\tFASTQ\tDNA\t1000000\t150000000\t50\t150.0\t300\t150\t150\t150\t0\t150\t1000000\t98.5\t95.2\t39.0\t50.0\t1000>>END_MODULE")?;

        let row = parse_fqc_data(temp_file.path().to_str().unwrap())?;
        assert_eq!(row.num_seqs, 1000000);
        assert_eq!(row.sum_len, 150000000);
        assert_eq!(row.q20, 98.5);
        assert_eq!(row.q30, 95.2);
        assert_eq!(row.min_len, 50);
        assert_eq!(row.avg_len, 150.0);
        assert_eq!(row.max_len, 300);

        Ok(())
    }
}
