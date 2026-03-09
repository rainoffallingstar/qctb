use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

// ── New nested-format intermediate structs ─────────────────────────────────

#[derive(Debug, Deserialize, Default)]
struct NestedQCDirsQC {
    #[serde(default)]
    pub main: String,
    #[serde(default)]
    pub after: Option<String>,
    #[serde(default)]
    pub before: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct NestedQCDirsBsmap {
    #[serde(default)]
    pub main: String,
}

#[derive(Debug, Deserialize, Default)]
struct NestedQCDirs {
    #[serde(default)]
    pub qc: NestedQCDirsQC,
    #[serde(default)]
    pub bsmap: NestedQCDirsBsmap,
    #[serde(default)]
    pub methylation_call: String,
}

#[derive(Debug, Deserialize, Default)]
struct NestedOutput {
    #[serde(default)]
    pub trim_dir: String,
}

#[derive(Debug, Deserialize, Default)]
struct NestedWorkflowSpecies {
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Deserialize, Default)]
struct NestedWorkflow {
    #[serde(default)]
    pub species: NestedWorkflowSpecies,
}

#[derive(Debug, Deserialize, Default)]
struct NestedMetadata {
    #[serde(default)]
    pub sample_ids: Vec<String>,
}

// ── Raw config: handles both old flat and new nested format ────────────────

#[derive(Debug, Deserialize)]
struct RawConfig {
    // Old flat-format fields (all optional for backwards-compat)
    #[serde(rename = "SIDs", default)]
    sids: Vec<String>,
    #[serde(default)]
    graft: Option<String>,
    #[serde(rename = "qcDir", default)]
    qc_dir: String,
    #[serde(rename = "trimDir", default)]
    trim_dir: String,
    #[serde(rename = "bsmapDir", default)]
    bsmap_dir: String,
    #[serde(rename = "outDir_mCall", default)]
    outdir_mcall: String,
    #[serde(rename = "qcDir_before", default)]
    qcdir_before: Option<String>,
    #[serde(rename = "qcDir_after", default)]
    qcdir_after: Option<String>,

    // New nested-format fields
    #[serde(default)]
    metadata: NestedMetadata,
    #[serde(default)]
    directories: NestedQCDirs,
    #[serde(default)]
    output: NestedOutput,
    #[serde(default)]
    workflow: NestedWorkflow,
}

// ── Public config returned to callers ─────────────────────────────────────

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct QCConfig {
    pub SIDs: Vec<String>,
    pub graft: Option<String>,
    pub qcDir: String,
    pub trimDir: String,
    pub bsmap_dir: String,
    pub outdir_mcall: String,
    pub qcdir_before: Option<String>,
    pub qcdir_after: Option<String>,
}

pub fn load_config(config_path: &Path) -> Result<QCConfig> {
    let file = std::fs::File::open(config_path)
        .with_context(|| format!("Failed to open config file: {}", config_path.display()))?;

    let raw: RawConfig = serde_yaml::from_reader(file)
        .with_context(|| format!("Failed to parse YAML config: {}", config_path.display()))?;

    // Prefer old flat fields; fall back to new nested fields when flat fields are empty.
    let sids = if !raw.sids.is_empty() {
        raw.sids
    } else {
        raw.metadata.sample_ids
    };

    let qc_dir = if !raw.qc_dir.is_empty() {
        raw.qc_dir
    } else {
        raw.directories.qc.main.clone()
    };

    let trim_dir = if !raw.trim_dir.is_empty() {
        raw.trim_dir
    } else {
        raw.output.trim_dir.clone()
    };

    let bsmap_dir = if !raw.bsmap_dir.is_empty() {
        raw.bsmap_dir
    } else {
        raw.directories.bsmap.main.clone()
    };

    let outdir_mcall = if !raw.outdir_mcall.is_empty() {
        raw.outdir_mcall
    } else {
        raw.directories.methylation_call.clone()
    };

    let graft = raw.graft.or_else(|| {
        if !raw.workflow.species.name.is_empty() {
            Some(raw.workflow.species.name.clone())
        } else {
            None
        }
    });

    let qcdir_before = raw.qcdir_before.or(raw.directories.qc.before);
    let qcdir_after = raw.qcdir_after.or(raw.directories.qc.after);

    Ok(QCConfig {
        SIDs: sids,
        graft,
        qcDir: qc_dir,
        trimDir: trim_dir,
        bsmap_dir,
        outdir_mcall,
        qcdir_before,
        qcdir_after,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_config_old_format() -> Result<()> {
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
        assert_eq!(config.qcDir, "/qc");

        Ok(())
    }

    #[test]
    fn test_load_config_new_format() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "metadata:")?;
        writeln!(temp_file, "  sample_ids:")?;
        writeln!(temp_file, "    - sample1")?;
        writeln!(temp_file, "    - sample2")?;
        writeln!(temp_file, "directories:")?;
        writeln!(temp_file, "  qc:")?;
        writeln!(temp_file, "    main: /qc")?;
        writeln!(temp_file, "  bsmap:")?;
        writeln!(temp_file, "    main: /bsmap")?;
        writeln!(temp_file, "  methylation_call: /mcall")?;
        writeln!(temp_file, "output:")?;
        writeln!(temp_file, "  trim_dir: /trim")?;
        writeln!(temp_file, "workflow:")?;
        writeln!(temp_file, "  species:")?;
        writeln!(temp_file, "    name: human")?;

        let config = load_config(temp_file.path())?;
        assert_eq!(config.SIDs.len(), 2);
        assert_eq!(config.SIDs[0], "sample1");
        assert_eq!(config.graft, Some("human".to_string()));
        assert_eq!(config.qcDir, "/qc");
        assert_eq!(config.trimDir, "/trim");
        assert_eq!(config.bsmap_dir, "/bsmap");

        Ok(())
    }
}
