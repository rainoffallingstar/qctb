use serde::Deserialize;
use std::path::Path;
use anyhow::{Context, Result};

#[derive(Debug, Deserialize)]
pub struct QCConfig {
    pub SIDs: Vec<String>,
    #[serde(default)]
    pub graft: Option<String>,
    pub qcDir: String,
    pub trimDir: String,
    #[serde(rename = "bsmapDir")]
    pub bsmap_dir: String,
    #[serde(rename = "outDir_mCall")]
    pub outdir_mcall: String,
    #[serde(rename = "qcDir_before")]
    #[serde(default)]
    pub qcdir_before: Option<String>,
    #[serde(rename = "qcDir_after")]
    #[serde(default)]
    pub qcdir_after: Option<String>,
}

pub fn load_config(config_path: &Path) -> Result<QCConfig> {
    let file = std::fs::File::open(config_path)
        .with_context(|| format!("Failed to open config file: {}", config_path.display()))?;

    let config: QCConfig = serde_yaml::from_reader(file)
        .with_context(|| format!("Failed to parse YAML config: {}", config_path.display()))?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_config() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "SIDs:")?;
        writeln!(temp_file, "  - sample1")?;
        writeln!(temp_file, "  - sample2")?;
        writeln!(temp_file, "graft: \"human\"")?;
        writeln!(temp_file, "qcDir: \"/qc\"")?;
        writeln!(temp_file, "trimDir: \"/trim\"")?;
        writeln!(temp_file, "bsmapDir: \"/bsmap\"")?;
        writeln!(temp_file, "outDir_mCall: \"/mcall\"")?;

        let config = load_config(temp_file.path())?;
        assert_eq!(config.SIDs.len(), 2);
        assert_eq!(config.SIDs[0], "sample1");
        assert_eq!(config.graft, Some("human".to_string()));

        Ok(())
    }
}
