use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct SeqkitStats {
    // Raw reads R1
    pub reads_raw_r1: u64,
    pub bases_raw_r1: u64,
    pub q20_raw_r1: f64,
    pub q30_raw_r1: f64,
    pub min_len_raw_r1: u32,
    pub avg_len_raw_r1: f64,
    pub max_len_raw_r1: u32,

    // Raw reads R2
    pub reads_raw_r2: u64,
    pub bases_raw_r2: u64,
    pub q20_raw_r2: f64,
    pub q30_raw_r2: f64,
    pub min_len_raw_r2: u32,
    pub avg_len_raw_r2: f64,
    pub max_len_raw_r2: u32,

    // Clean reads R1
    pub reads_clean_r1: u64,
    pub bases_clean_r1: u64,
    pub q20_clean_r1: f64,
    pub q30_clean_r1: f64,
    pub min_len_clean_r1: u32,
    pub avg_len_clean_r1: f64,
    pub max_len_clean_r1: u32,

    // Clean reads R2
    pub reads_clean_r2: u64,
    pub bases_clean_r2: u64,
    pub q20_clean_r2: f64,
    pub q30_clean_r2: f64,
    pub min_len_clean_r2: u32,
    pub avg_len_clean_r2: f64,
    pub max_len_clean_r2: u32,

    // Aggregated
    pub reads_raw: u64,
    pub bases_raw: u64,
    pub reads_clean: u64,
    pub bases_clean: u64,
    pub clean_data_ratio: f64,
}

pub fn parse_seqkit(file_path: &str) -> Result<SeqkitStats> {
    let file = File::open(file_path)
        .with_context(|| format!("Failed to open seqkit file: {}", file_path))?;

    let reader = BufReader::new(file);
    // Collect non-empty, non-comment lines (skip enva banner lines starting with '#')
    let all_lines: Vec<String> = reader.lines()
        .collect::<std::io::Result<Vec<String>>>()
        .with_context(|| format!("Failed to read seqkit file: {}", file_path))?;
    let lines: Vec<String> = all_lines.into_iter()
        .filter(|l: &String| {
            let trimmed = l.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .collect();

    if lines.len() < 5 {
        anyhow::bail!("Seqkit file has too few data rows (expected ≥5, got {}): {}", lines.len(), file_path);
    }

    // Parse header to find column indices
    let header = &lines[0];
    let headers: Vec<&str> = header.split_whitespace().collect();

    let find_col = |names: &[&str]| -> Result<usize> {
        for name in names {
            if let Some(pos) = headers.iter().position(|&h| h == *name) {
                return Ok(pos);
            }
        }
        anyhow::bail!("Column(s) {:?} not found in seqkit file header: {}", names, file_path)
    };

    let col_num_seqs = find_col(&["num_seqs"])?;
    let col_sum_len  = find_col(&["sum_len"])?;
    // Accept both Q20(%) (newer seqkit) and Q20... (older seqkit)
    let col_q20      = find_col(&["Q20(%)", "Q20..."])?;
    let col_q30      = find_col(&["Q30(%)", "Q30..."])?;
    let col_min_len  = find_col(&["min_len"])?;
    let col_avg_len  = find_col(&["avg_len"])?;
    let col_max_len  = find_col(&["max_len"])?;

    let parse_row = |line: &str| -> Result<Vec<f64>> {
        let cols: Vec<&str> = line.split_whitespace().collect();
        let get_value = |idx: usize| -> Result<f64> {
            cols.get(idx)
                .with_context(|| format!("Column index {} out of bounds", idx))?
                .parse::<f64>()
                .with_context(|| format!("Failed to parse value at column {}", idx))
        };
        Ok(vec![
            get_value(col_num_seqs)?,
            get_value(col_sum_len)?,
            get_value(col_q20)?,
            get_value(col_q30)?,
            get_value(col_min_len)?,
            get_value(col_avg_len)?,
            get_value(col_max_len)?,
        ])
    };

    // Row 1: R1 raw, Row 2: R2 raw, Row 3: R1 clean, Row 4: R2 clean
    let row1 = parse_row(&lines[1])?;
    let row2 = parse_row(&lines[2])?;
    let row3 = parse_row(&lines[3])?;
    let row4 = parse_row(&lines[4])?;

    let reads_raw = (row1[0] + row2[0]) as u64;
    let bases_raw = (row1[1] + row2[1]) as u64;
    let reads_clean = (row3[0] + row4[0]) as u64;
    let bases_clean = (row3[1] + row4[1]) as u64;
    let clean_data_ratio = if bases_raw > 0 {
        (bases_clean as f64) / (bases_raw as f64)
    } else {
        0.0
    };

    Ok(SeqkitStats {
        reads_raw_r1: row1[0] as u64,
        bases_raw_r1: row1[1] as u64,
        q20_raw_r1: row1[2],
        q30_raw_r1: row1[3],
        min_len_raw_r1: row1[4] as u32,
        avg_len_raw_r1: row1[5],
        max_len_raw_r1: row1[6] as u32,

        reads_raw_r2: row2[0] as u64,
        bases_raw_r2: row2[1] as u64,
        q20_raw_r2: row2[2],
        q30_raw_r2: row2[3],
        min_len_raw_r2: row2[4] as u32,
        avg_len_raw_r2: row2[5],
        max_len_raw_r2: row2[6] as u32,

        reads_clean_r1: row3[0] as u64,
        bases_clean_r1: row3[1] as u64,
        q20_clean_r1: row3[2],
        q30_clean_r1: row3[3],
        min_len_clean_r1: row3[4] as u32,
        avg_len_clean_r1: row3[5],
        max_len_clean_r1: row3[6] as u32,

        reads_clean_r2: row4[0] as u64,
        bases_clean_r2: row4[1] as u64,
        q20_clean_r2: row4[2],
        q30_clean_r2: row4[3],
        min_len_clean_r2: row4[4] as u32,
        avg_len_clean_r2: row4[5],
        max_len_clean_r2: row4[6] as u32,

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
    fn test_parse_seqkit() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "num_seqs\tsum_len\tQ20...\tQ30...\tmin_len\tavg_len\tmax_len")?;
        writeln!(temp_file, "1000000\t150000000\t98.5\t95.2\t50\t150\t300")?;
        writeln!(temp_file, "1000000\t150000000\t98.3\t95.0\t50\t150\t300")?;
        writeln!(temp_file, "950000\t142500000\t99.0\t96.5\t50\t150\t300")?;
        writeln!(temp_file, "950000\t142500000\t98.8\t96.3\t50\t150\t300")?;

        let stats = parse_seqkit(temp_file.path().to_str().unwrap())?;
        assert_eq!(stats.reads_raw, 2000000);
        assert_eq!(stats.bases_raw, 300000000);
        assert_eq!(stats.reads_clean, 1900000);
        assert_eq!(stats.bases_clean, 285000000);

        Ok(())
    }
}
