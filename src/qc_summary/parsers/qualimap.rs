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
    let re_duplicated =
        Regex::new(r"number of duplicated reads(?:\s*\(estimated\))?\s*=\s*([\d,]+)")?;
    let re_duplication = Regex::new(r"duplication rate\s*=\s*([\d.]+)")?;

    let extract = |regex: &Regex, field: &str| -> Result<String> {
        regex
            .captures(&content)
            .and_then(|captures| captures.get(1))
            .map(|value| value.as_str().replace(',', ""))
            .with_context(|| {
                format!(
                    "Missing required Qualimap field '{}' in {}",
                    field, file_path
                )
            })
    };

    let mapping_quality = extract(&re_mapping_quality, "mean mapping quality")?;
    let duplicated_reads = extract(&re_duplicated, "number of duplicated reads")?;
    let duplication_ratio = extract(&re_duplication, "duplication rate")?;

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

    #[test]
    fn test_missing_required_field_fails() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "mean mapping quality = 60")?;
        writeln!(temp_file, "number of duplicated reads = 50000")?;
        assert!(parse_qualimap_report(temp_file.path().to_str().unwrap()).is_err());
        Ok(())
    }
}
