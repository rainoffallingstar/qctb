use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashSet;
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
    #[serde(default)]
    pub qualimap: String,
}

#[derive(Debug, Deserialize, Default)]
struct NestedOutput {
    #[serde(default)]
    pub trim_dir: String,
}

#[derive(Debug, Deserialize, Default)]
struct NestedWorkflowSpecies {
    #[serde(rename = "name", default)]
    _name: Vec<String>,
    #[serde(default)]
    graft: String,
}

#[derive(Debug, Deserialize, Default)]
struct NestedWorkflow {
    #[serde(default)]
    pub mode: String,
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
    #[serde(default)]
    mode: Option<String>,
    #[serde(rename = "qcDir", default)]
    qc_dir: String,
    #[serde(rename = "trimDir", default)]
    trim_dir: String,
    #[serde(rename = "bsmapDir", default)]
    bsmap_dir: String,
    #[serde(rename = "outDir_mCall", default)]
    outdir_mcall: String,
    #[serde(rename = "outdir_qualimap", default)]
    qualimap_dir: String,
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
    pub workflow_mode: Option<String>,
    pub qcDir: String,
    pub trimDir: String,
    pub bsmap_dir: String,
    pub qualimap_dir: String,
    pub outdir_mcall: String,
    pub qcdir_before: Option<String>,
    pub qcdir_after: Option<String>,
}

fn validate_sample_ids(sample_ids: &[String]) -> Result<()> {
    if sample_ids.is_empty() {
        anyhow::bail!("Configuration must contain at least one sample ID");
    }
    let mut seen = HashSet::new();
    for sample_id in sample_ids {
        if sample_id.trim().is_empty() {
            anyhow::bail!("Sample IDs must not be empty or whitespace-only");
        }
        if sample_id != sample_id.trim() {
            anyhow::bail!("Sample ID must not have leading or trailing whitespace: {sample_id:?}");
        }
        if sample_id.contains(['\t', '\r', '\n']) {
            anyhow::bail!("Sample ID contains a tab or newline: {sample_id:?}");
        }
        if !seen.insert(sample_id) {
            anyhow::bail!("Duplicate sample ID in configuration: '{sample_id}'");
        }
    }
    Ok(())
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

    let qualimap_dir = if !raw.qualimap_dir.is_empty() {
        raw.qualimap_dir
    } else {
        raw.directories.qualimap.clone()
    };
    let outdir_mcall = if !raw.outdir_mcall.is_empty() {
        raw.outdir_mcall
    } else {
        raw.directories.methylation_call.clone()
    };

    let workflow_mode = raw.mode.or(if raw.workflow.mode.trim().is_empty() {
        None
    } else {
        Some(raw.workflow.mode)
    });
    let graft = raw.graft.or(if raw.workflow.species.graft.is_empty() {
        None
    } else {
        Some(raw.workflow.species.graft)
    });

    let qcdir_before = raw.qcdir_before.or(raw.directories.qc.before);
    let qcdir_after = raw.qcdir_after.or(raw.directories.qc.after);
    validate_sample_ids(&sids)?;

    Ok(QCConfig {
        SIDs: sids,
        graft,
        workflow_mode,
        qcDir: qc_dir,
        trimDir: trim_dir,
        bsmap_dir,
        qualimap_dir,
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
    fn test_load_config_main_repository_nested_format() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(
            br#"SIDs:
    - tumor_sample
    - normal_sample
directories:
    bsmap:
        main: /analysis/workflow/bsmap
    methylation_call: /analysis/workflow/mCall
    qualimap: /analysis/custom/qualimap
    qc:
        after: /analysis/workflow/fastqc_clean
        before: /analysis/workflow/fastqc_raw
        main: /analysis/workflow/QC
metadata:
    sample_ids:
        - tumor_sample
        - normal_sample
output:
    trim_dir: /analysis/workflow/trim
workflow:
    mode: PDX
    species:
        graft: human
        host: mouse
        name:
            - human
            - mouse
"#,
        )?;

        let config = load_config(temp_file.path())?;
        assert_eq!(
            config.SIDs,
            vec!["tumor_sample".to_string(), "normal_sample".to_string()]
        );
        assert_eq!(config.graft.as_deref(), Some("human"));
        assert_eq!(config.workflow_mode.as_deref(), Some("PDX"));
        assert_eq!(config.qcDir, "/analysis/workflow/QC");
        assert_eq!(
            config.qcdir_before.as_deref(),
            Some("/analysis/workflow/fastqc_raw")
        );
        assert_eq!(
            config.qcdir_after.as_deref(),
            Some("/analysis/workflow/fastqc_clean")
        );
        assert_eq!(config.trimDir, "/analysis/workflow/trim");
        assert_eq!(config.bsmap_dir, "/analysis/workflow/bsmap");
        assert_eq!(config.qualimap_dir, "/analysis/custom/qualimap");
        assert_eq!(config.outdir_mcall, "/analysis/workflow/mCall");

        Ok(())
    }

    #[test]
    fn empty_duplicate_and_whitespace_sample_ids_are_rejected() -> Result<()> {
        for yaml in [
            "SIDs: []\n",
            "SIDs:\n  - sample1\n  - sample1\n",
            "SIDs:\n  - ' sample1'\n",
            "SIDs:\n  - '   '\n",
        ] {
            let mut temp_file = NamedTempFile::new()?;
            temp_file.write_all(yaml.as_bytes())?;
            assert!(
                load_config(temp_file.path()).is_err(),
                "configuration should be rejected: {yaml:?}"
            );
        }
        Ok(())
    }
}
