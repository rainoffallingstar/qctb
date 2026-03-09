use crate::qc_summary::{parsers::*, QCConfig};
use anyhow::{bail, Context, Result};
use std::path::Path;

#[derive(Debug)]
pub struct QCSummary {
    pub sample_id: String,
    pub seqkit_stats: SeqkitStats,
    pub trim_stats: TrimStats,
    pub bismark_stats: Option<BismarkStats>,
    pub qualimap_stats: Option<QualimapStats>,
    pub methrix_coverage: Option<MethrixCoverageRow>,
    pub methrix_annotation: Option<MethrixAnnotationBySampleRow>,
}

#[derive(Debug)]
pub struct QCSummaryRNA {
    pub sample_id: String,
    pub seqkit_stats: SeqkitStats,
    pub trim_stats: TrimStats,
    pub star_stats: StarStats,
}

fn require_existing_path(label: &str, sid: &str, candidates: &[String]) -> Result<String> {
    for p in candidates {
        if Path::new(p).exists() {
            return Ok(p.clone());
        }
    }
    bail!(
        "Missing required {} file for sample '{}'. Tried: {}",
        label,
        sid,
        candidates.join(", ")
    )
}

fn parse_optional_methrix_coverage(
    config: &QCConfig,
    sid: &str,
) -> Result<Option<MethrixCoverageRow>> {
    if config.outdir_mcall.is_empty() {
        return Ok(None);
    }
    let candidates = [
        format!(
            "{}/CpG_coverage_recomputed_from_h5.xlsx",
            config.outdir_mcall
        ),
        format!("{}/CpG_coverage.xlsx", config.outdir_mcall),
    ];
    let path = candidates
        .iter()
        .find(|p| Path::new(p).exists())
        .map(|s| s.as_str());
    let Some(path) = path else {
        return Ok(None);
    };

    let rows = parse_methrix_coverage_xlsx(path)
        .with_context(|| format!("Failed to parse methrix coverage report: {}", path))?;
    Ok(rows.into_iter().find(|r| {
        r.sample == sid
            || r.sample.starts_with(sid)
            || r.sample.contains(&format!("{}_", sid))
            || r.sample.contains(sid)
    }))
}

fn parse_optional_methrix_annotation(
    config: &QCConfig,
    sid: &str,
) -> Result<Option<MethrixAnnotationBySampleRow>> {
    if config.outdir_mcall.is_empty() {
        return Ok(None);
    }
    let path = format!("{}/CpG_annotation_report.xlsx", config.outdir_mcall);
    if !Path::new(&path).exists() {
        return Ok(None);
    }

    let rows = parse_methrix_annotation_by_sample_xlsx(&path)
        .with_context(|| format!("Failed to parse methrix annotation report: {}", path))?;
    Ok(rows.into_iter().find(|r| {
        r.sample == sid
            || r.sample.starts_with(sid)
            || r.sample.contains(&format!("{}_", sid))
            || r.sample.contains(sid)
    }))
}

pub fn process_sample(config: &QCConfig, sid: &str) -> Result<QCSummary> {
    // Parse fqc fastqc_data.txt files (replaces seqkit)
    let before = config.qcdir_before.as_deref().unwrap_or(&config.qcDir);
    let after = config.qcdir_after.as_deref().unwrap_or(&config.qcDir);
    let raw_r1 = format!("{}/{}_R1_fqc/fastqc_data.txt", before, sid);
    let raw_r2 = format!("{}/{}_R2_fqc/fastqc_data.txt", before, sid);
    let clean_r1 = format!("{}/{}_val_1_fqc/fastqc_data.txt", after, sid);
    let clean_r2 = format!("{}/{}_val_2_fqc/fastqc_data.txt", after, sid);
    let seqkit_stats = parse_seqkit_from_fqc(&raw_r1, &raw_r2, &clean_r1, &clean_r2)
        .with_context(|| format!("Failed to parse fqc stats for sample: {}", sid))?;

    // Parse trim galore files (R1 and R2)
    let trim_r1_file = format!("{}/{}_R1.fastq.gz_trimming_report.txt", config.trimDir, sid);
    let trim_r2_file = format!("{}/{}_R2.fastq.gz_trimming_report.txt", config.trimDir, sid);
    let trim_stats = parse_trim_reports(&trim_r1_file, &trim_r2_file)
        .with_context(|| format!("Failed to parse trim galore files for sample: {}", sid))?;

    // Parse bismark report (required)
    let graft = config.graft.as_deref().unwrap_or("human");
    let bismark_candidates = vec![
        format!(
            "{}/{}/{}_val_1_bismark_bt2_PE_report.txt",
            config.bsmap_dir, graft, sid
        ),
        format!(
            "{}/{}_val_1_bismark_bt2_PE_report.txt",
            config.bsmap_dir, sid
        ),
    ];
    let bismark_file = require_existing_path("bismark", sid, &bismark_candidates)?;
    let bismark_stats = parse_bismark_report(&bismark_file)
        .with_context(|| format!("Failed to parse bismark file for sample: {}", sid))?;

    // Parse qualimap report (required)
    let qualimap_candidates = vec![
        format!(
            "{}/qualimap/{}_{}/genome_results.txt",
            config.qcDir, sid, graft
        ),
        format!("{}/{}_{}/genome_results.txt", config.qcDir, sid, graft),
    ];
    let qualimap_results_file = require_existing_path("qualimap", sid, &qualimap_candidates)?;
    let qualimap_stats = parse_qualimap_report(&qualimap_results_file)
        .with_context(|| format!("Failed to parse qualimap file for sample: {}", sid))?;

    let methrix_coverage = parse_optional_methrix_coverage(config, sid)?;
    let methrix_annotation = parse_optional_methrix_annotation(config, sid)?;

    Ok(QCSummary {
        sample_id: sid.to_string(),
        seqkit_stats,
        trim_stats,
        bismark_stats: Some(bismark_stats),
        qualimap_stats: Some(qualimap_stats),
        methrix_coverage,
        methrix_annotation,
    })
}

pub fn process_sample_rnaseq(config: &QCConfig, sid: &str) -> Result<QCSummaryRNA> {
    // Parse fqc fastqc_data.txt files (replaces seqkit)
    let before = config.qcdir_before.as_deref().unwrap_or(&config.qcDir);
    let after = config.qcdir_after.as_deref().unwrap_or(&config.qcDir);
    let raw_r1 = format!("{}/{}_R1_fqc/fastqc_data.txt", before, sid);
    let raw_r2 = format!("{}/{}_R2_fqc/fastqc_data.txt", before, sid);
    let clean_r1 = format!("{}/{}_val_1_fqc/fastqc_data.txt", after, sid);
    let clean_r2 = format!("{}/{}_val_2_fqc/fastqc_data.txt", after, sid);
    let seqkit_stats = parse_seqkit_from_fqc(&raw_r1, &raw_r2, &clean_r1, &clean_r2)
        .with_context(|| format!("Failed to parse fqc stats for sample: {}", sid))?;

    // Parse trim galore files (R1 and R2)
    let trim_r1_file = format!("{}/{}_R1.fastq.gz_trimming_report.txt", config.trimDir, sid);
    let trim_r2_file = format!("{}/{}_R2.fastq.gz_trimming_report.txt", config.trimDir, sid);
    let trim_stats = parse_trim_reports(&trim_r1_file, &trim_r2_file)
        .with_context(|| format!("Failed to parse trim galore files for sample: {}", sid))?;

    // Parse STAR log file
    let graft = config.graft.as_deref().unwrap_or("human");
    let star_candidates = vec![
        format!("{}/{}/{}Log.final.out", config.bsmap_dir, graft, sid),
        format!("{}/{}Log.final.out", config.bsmap_dir, sid),
    ];
    let star_file = require_existing_path("STAR Log.final.out", sid, &star_candidates)?;
    let star_stats = parse_star_log(&star_file)
        .with_context(|| format!("Failed to parse STAR log file for sample: {}", sid))?;

    Ok(QCSummaryRNA {
        sample_id: sid.to_string(),
        seqkit_stats,
        trim_stats,
        star_stats,
    })
}

pub fn process_all_samples(config: &QCConfig) -> Result<Vec<QCSummary>> {
    let mut summaries = Vec::new();

    for sid in &config.SIDs {
        let summary = process_sample(config, sid)
            .with_context(|| format!("Failed to process sample '{}'", sid))?;
        summaries.push(summary);
    }

    Ok(summaries)
}

pub fn process_all_samples_rnaseq(config: &QCConfig) -> Result<Vec<QCSummaryRNA>> {
    let mut summaries = Vec::new();

    for sid in &config.SIDs {
        let summary = process_sample_rnaseq(config, sid)
            .with_context(|| format!("Failed to process sample '{}'", sid))?;
        summaries.push(summary);
    }

    Ok(summaries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn repo_testdata() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata")
    }

    fn mk_config_standard(bsmap_dir: &str) -> QCConfig {
        let td = repo_testdata();
        QCConfig {
            SIDs: vec!["F6703_372760".to_string()],
            graft: Some("human".to_string()),
            qcDir: td.to_string_lossy().to_string(),
            trimDir: td.join("trim").to_string_lossy().to_string(),
            bsmap_dir: bsmap_dir.to_string(),
            outdir_mcall: String::new(),
            qcdir_before: Some(td.join("fqc_raw").to_string_lossy().to_string()),
            qcdir_after: Some(td.join("fqc_clean").to_string_lossy().to_string()),
        }
    }

    fn mk_config_rnaseq() -> QCConfig {
        let td = repo_testdata();
        QCConfig {
            SIDs: vec!["F6703_372760".to_string()],
            graft: Some("human".to_string()),
            qcDir: td.to_string_lossy().to_string(),
            trimDir: td.join("trim").to_string_lossy().to_string(),
            bsmap_dir: td.join("star").to_string_lossy().to_string(),
            outdir_mcall: String::new(),
            qcdir_before: Some(td.join("fqc_raw").to_string_lossy().to_string()),
            qcdir_after: Some(td.join("fqc_clean").to_string_lossy().to_string()),
        }
    }

    #[test]
    fn test_process_all_samples_realworld_standard_success() -> Result<()> {
        let temp = TempDir::new()?;
        let sid = "F6703_372760";
        let src = repo_testdata()
            .join("bismark")
            .join("0531LPHSC70203_val_1_bismark_bt2_PE_report.txt");
        let dst = temp
            .path()
            .join(format!("{}_val_1_bismark_bt2_PE_report.txt", sid));
        fs::copy(src, dst)?;

        let cfg = mk_config_standard(temp.path().to_string_lossy().as_ref());
        let summaries = process_all_samples(&cfg)?;
        assert_eq!(summaries.len(), 1);
        let s = &summaries[0];

        assert_eq!(s.sample_id, sid);
        assert_eq!(s.seqkit_stats.reads_raw, 48_130_550);
        assert_eq!(s.seqkit_stats.reads_clean, 47_108_944);

        let bs = s
            .bismark_stats
            .as_ref()
            .expect("bismark stats should exist");
        assert_eq!(bs.mapping_ratio, "63.3%");
        assert_eq!(bs.total_reads_pairs, "15352914");
        assert_eq!(bs.aligned_reads_pairs, "9721219");
        assert!((bs.aligned_reads_pairs_ratio - 0.6332).abs() < 0.0001);

        let q = s
            .qualimap_stats
            .as_ref()
            .expect("qualimap stats should exist");
        assert_eq!(q.mapping_quality, "15.6988");
        assert_eq!(q.duplication_ratio, "51.22");
        Ok(())
    }

    #[test]
    fn test_process_all_samples_realworld_missing_bismark_fails() {
        let cfg = mk_config_standard(repo_testdata().join("bismark").to_string_lossy().as_ref());
        let err = process_all_samples(&cfg).expect_err("expected missing bismark to fail");
        let msg = format!("{err:#}");
        assert!(msg.contains("Missing required bismark file"));
        assert!(msg.contains("F6703_372760"));
    }

    #[test]
    fn test_process_all_samples_realworld_rnaseq_success() -> Result<()> {
        let cfg = mk_config_rnaseq();
        let summaries = process_all_samples_rnaseq(&cfg)?;
        assert_eq!(summaries.len(), 1);
        let s = &summaries[0];
        assert_eq!(s.sample_id, "F6703_372760");
        assert_eq!(s.seqkit_stats.reads_raw, 48_130_550);
        assert_eq!(s.seqkit_stats.reads_clean, 47_108_944);
        assert_eq!(s.star_stats.mapping_ratio, "82.63%");
        assert_eq!(s.star_stats.total_reads, "23554472");
        assert_eq!(s.star_stats.uniquely_mapped_reads, "19463458");
        assert!((s.star_stats.uniquely_mapped_ratio - 0.8263).abs() < 0.0001);
        Ok(())
    }

    #[test]
    fn test_process_all_samples_realworld_rnaseq_missing_star_fails() {
        let td = repo_testdata();
        let mut cfg = mk_config_rnaseq();
        cfg.bsmap_dir = td.join("bismark").to_string_lossy().to_string();
        let err = process_all_samples_rnaseq(&cfg).expect_err("expected missing STAR to fail");
        let msg = format!("{err:#}");
        assert!(msg.contains("Missing required STAR Log.final.out file"));
        assert!(msg.contains("F6703_372760"));
    }
}
