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

    let extract = |regex: &Regex, field: &str| -> Result<String> {
        regex
            .captures(&content)
            .and_then(|captures| captures.get(1))
            .map(|value| value.as_str().replace(',', ""))
            .with_context(|| {
                format!(
                    "Missing required Bismark field '{}' in {}",
                    field, file_path
                )
            })
    };

    let mapping_value = extract(&re_mapping, "Mapping efficiency")?;
    let mapping_ratio = format!("{}%", mapping_value);
    let total_reads_pairs = extract(&re_total, "Sequence pairs analysed in total")?;
    let aligned_reads_pairs = extract(
        &re_aligned,
        "Number of paired-end alignments with a unique best hit",
    )?;

    let total_num: u64 = total_reads_pairs
        .parse()
        .with_context(|| format!("Invalid total read-pair count in {}", file_path))?;
    let aligned_num: u64 = aligned_reads_pairs
        .parse()
        .with_context(|| format!("Invalid aligned read-pair count in {}", file_path))?;
    if total_num == 0 {
        anyhow::bail!("Bismark total read-pair count is zero in {}", file_path);
    }
    let aligned_reads_pairs_ratio = (aligned_num as f64) / (total_num as f64);

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

    #[test]
    fn test_missing_required_field_fails() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "Mapping efficiency: 95.23%")?;
        writeln!(temp_file, "Sequence pairs analysed in total: 1000000")?;
        assert!(parse_bismark_report(temp_file.path().to_str().unwrap()).is_err());
        Ok(())
    }
}
