use anyhow::{Context, Result};
use regex::Regex;
use std::fs::read_to_string;

#[derive(Debug, Clone)]
pub struct QualimapStats {
    pub mapping_quality: String,
    pub duplicated_reads: String,
    pub duplication_ratio: String,
}

pub fn parse_qualimap_report(file_path: &str) -> Result<QualimapStats> {
    let content = read_to_string(file_path)
        .with_context(|| format!("Failed to read qualimap report: {}", file_path))?;

    // Parse qualimap genome_results.txt format
    // Example lines:
    // mean mapping quality = 60
    // number of duplicated reads = 50000
    // duplication rate = 0.0526

    let re_mapping_quality = Regex::new(r"mean mapping quality\s*=\s*([\d.]+)")?;
    let re_duplicated = Regex::new(r"number of duplicated reads\s*=\s*([\d,]+)")?;
    let re_duplication = Regex::new(r"duplication rate\s*=\s*([\d.]+)")?;

    let mapping_quality = re_mapping_quality
        .captures(&content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "N/A".to_string());

    let duplicated_reads = re_duplicated
        .captures(&content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().replace(',', ""))
        .unwrap_or_else(|| "0".to_string());

    let duplication_ratio = re_duplication
        .captures(&content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "0.0".to_string());

    Ok(QualimapStats {
        mapping_quality,
        duplicated_reads,
        duplication_ratio,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_qualimap_report() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "qualimap report")?;
        writeln!(temp_file, "mean mapping quality = 60")?;
        writeln!(temp_file, "number of duplicated reads = 50000")?;
        writeln!(temp_file, "duplication rate = 0.0526")?;

        let stats = parse_qualimap_report(temp_file.path().to_str().unwrap())?;
        assert_eq!(stats.mapping_quality, "60");
        assert_eq!(stats.duplicated_reads, "50000");
        assert_eq!(stats.duplication_ratio, "0.0526");

        Ok(())
    }
}
