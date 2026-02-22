use anyhow::{Context, Result};
use regex::Regex;
use std::fs::read_to_string;

#[derive(Debug, Clone)]
pub struct StarStats {
    pub mapping_ratio: String,
    pub total_reads: String,
    pub uniquely_mapped_reads: String,
    pub uniquely_mapped_ratio: f64,
}

pub fn parse_star_log(file_path: &str) -> Result<StarStats> {
    let content = read_to_string(file_path)
        .with_context(|| format!("Failed to read STAR log file: {}", file_path))?;

    // Parse STAR Log.final.out format
    // Example lines:
    // | Uniquely mapped reads % | 95.23% |
    // | Number of input reads | 1000000 |
    // | Uniquely mapped reads number | 952300 |

    let re_uniquely_pct = Regex::new(r"Uniquely mapped reads %\s*\|\s*([\d.]+)%")?;
    let re_input_reads = Regex::new(r"Number of input reads\s*\|\s*([\d,]+)")?;
    let re_uniquely_number = Regex::new(r"Uniquely mapped reads number\s*\|\s*([\d,]+)")?;

    let mapping_ratio = re_uniquely_pct
        .captures(&content)
        .and_then(|c| c.get(1))
        .map(|m| format!("{}%", m.as_str()))
        .unwrap_or_else(|| "N/A".to_string());

    let total_reads = re_input_reads
        .captures(&content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().replace(',', ""))
        .unwrap_or_else(|| "0".to_string());

    let uniquely_mapped_reads = re_uniquely_number
        .captures(&content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().replace(',', ""))
        .unwrap_or_else(|| "0".to_string());

    let total_reads_num: u64 = total_reads.parse().unwrap_or(0);
    let uniquely_mapped_num: u64 = uniquely_mapped_reads.parse().unwrap_or(0);

    let uniquely_mapped_ratio = if total_reads_num > 0 {
        (uniquely_mapped_num as f64) / (total_reads_num as f64)
    } else {
        0.0
    };

    Ok(StarStats {
        mapping_ratio,
        total_reads,
        uniquely_mapped_reads,
        uniquely_mapped_ratio,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_star_log() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "STAR Log.final.out")?;
        writeln!(temp_file, "Started job on")?;
        writeln!(temp_file, "| Uniquely mapped reads % | 95.23% |")?;
        writeln!(temp_file, "| Number of input reads | 1000000 |")?;
        writeln!(temp_file, "| Uniquely mapped reads number | 952300 |")?;
        writeln!(temp_file, "Finished job on")?;

        let stats = parse_star_log(temp_file.path().to_str().unwrap())?;
        assert_eq!(stats.mapping_ratio, "95.23%");
        assert_eq!(stats.total_reads, "1000000");
        assert_eq!(stats.uniquely_mapped_reads, "952300");
        assert!((stats.uniquely_mapped_ratio - 0.9523).abs() < 0.0001);

        Ok(())
    }

    #[test]
    fn test_parse_star_log_with_commas() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "STAR Log.final.out")?;
        writeln!(temp_file, "| Uniquely mapped reads % | 92.15% |")?;
        writeln!(temp_file, "| Number of input reads | 50,000,000 |")?;
        writeln!(temp_file, "| Uniquely mapped reads number | 46,075,000 |")?;

        let stats = parse_star_log(temp_file.path().to_str().unwrap())?;
        assert_eq!(stats.mapping_ratio, "92.15%");
        assert_eq!(stats.total_reads, "50000000");
        assert_eq!(stats.uniquely_mapped_reads, "46075000");

        Ok(())
    }
}
