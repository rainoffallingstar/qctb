use anyhow::{Context, Result};
use regex::Regex;
use std::fs::read_to_string;

#[derive(Debug, Clone)]
pub struct BismarkStats {
    pub mapping_ratio: String,
    pub total_reads_pairs: String,
    pub aligned_reads_pairs: String,
    pub aligned_reads_pairs_ratio: f64,
}

pub fn parse_bismark_report(file_path: &str) -> Result<BismarkStats> {
    let content = read_to_string(file_path)
        .with_context(|| format!("Failed to read bismark report: {}", file_path))?;

    // Parse bismark PE report format
    // Example lines:
    // Mapping efficiency: 95.23%
    // Sequence pairs analysed in total: 1000000
    // Number of paired-end alignments with a unique best hit: 952300

    let re_mapping = Regex::new(r"Mapping efficiency:\s*([\d.]+)%")?;
    let re_total = Regex::new(r"Sequence pairs analysed in total:\s+([\d,]+)")?;
    let re_aligned =
        Regex::new(r"Number of paired-end alignments with a unique best hit:\s+([\d,]+)")?;

    let mapping_ratio = re_mapping
        .captures(&content)
        .and_then(|c| c.get(1))
        .map(|m| format!("{}%", m.as_str()))
        .unwrap_or_else(|| "N/A".to_string());

    let total_reads_pairs = re_total
        .captures(&content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().replace(',', ""))
        .unwrap_or_else(|| "0".to_string());

    let aligned_reads_pairs = re_aligned
        .captures(&content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().replace(',', ""))
        .unwrap_or_else(|| "0".to_string());

    let total_num: u64 = total_reads_pairs.parse().unwrap_or(0);
    let aligned_num: u64 = aligned_reads_pairs.parse().unwrap_or(0);

    let aligned_reads_pairs_ratio = if total_num > 0 {
        (aligned_num as f64) / (total_num as f64)
    } else {
        0.0
    };

    Ok(BismarkStats {
        mapping_ratio,
        total_reads_pairs,
        aligned_reads_pairs,
        aligned_reads_pairs_ratio,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_bismark_report() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "Bismark PE report")?;
        writeln!(temp_file, "Mapping efficiency: 95.23%")?;
        writeln!(temp_file, "Sequence pairs analysed in total: 1000000")?;
        writeln!(
            temp_file,
            "Number of paired-end alignments with a unique best hit: 952300"
        )?;

        let stats = parse_bismark_report(temp_file.path().to_str().unwrap())?;
        assert_eq!(stats.mapping_ratio, "95.23%");
        assert_eq!(stats.total_reads_pairs, "1000000");
        assert_eq!(stats.aligned_reads_pairs, "952300");
        assert!((stats.aligned_reads_pairs_ratio - 0.9523).abs() < 0.0001);

        Ok(())
    }
}
