use crate::qc_summary::{QCConfig, parsers::*};
use anyhow::{Context, Result};

#[derive(Debug)]
pub struct QCSummary {
    pub sample_id: String,
    pub seqkit_stats: SeqkitStats,
    pub trim_stats: TrimStats,
    pub bismark_stats: Option<BismarkStats>,
    pub qualimap_stats: Option<QualimapStats>,
}

#[derive(Debug)]
pub struct QCSummaryRNA {
    pub sample_id: String,
    pub seqkit_stats: SeqkitStats,
    pub trim_stats: TrimStats,
    pub star_stats: StarStats,
}

pub fn process_sample(config: &QCConfig, sid: &str) -> Result<QCSummary> {
    // Parse fqc fastqc_data.txt files (replaces seqkit)
    let before = config.qcdir_before.as_deref().unwrap_or(&config.qcDir);
    let after  = config.qcdir_after.as_deref().unwrap_or(&config.qcDir);
    let raw_r1   = format!("{}/{}_R1_fqc/fastqc_data.txt", before, sid);
    let raw_r2   = format!("{}/{}_R2_fqc/fastqc_data.txt", before, sid);
    let clean_r1 = format!("{}/{}_val_1_fqc/fastqc_data.txt", after, sid);
    let clean_r2 = format!("{}/{}_val_2_fqc/fastqc_data.txt", after, sid);
    let seqkit_stats = parse_seqkit_from_fqc(&raw_r1, &raw_r2, &clean_r1, &clean_r2)
        .with_context(|| format!("Failed to parse fqc stats for sample: {}", sid))?;

    // Parse trim galore files (R1 and R2)
    let trim_r1_file = format!("{}/{}_R1.fastq.gz_trimming_report.txt", config.trimDir, sid);
    let trim_r2_file = format!("{}/{}_R2.fastq.gz_trimming_report.txt", config.trimDir, sid);
    let trim_stats = parse_trim_reports(&trim_r1_file, &trim_r2_file)
        .with_context(|| format!("Failed to parse trim galore files for sample: {}", sid))?;

    // Parse bismark report (optional)
    let graft = config.graft.as_deref().unwrap_or("human");
    let bismark_file = format!("{}/{}/{}_val_1_bismark_bt2_PE_report.txt", config.bsmap_dir, graft, sid);
    let bismark_stats = match std::path::Path::new(&bismark_file).exists() {
        true => Some(parse_bismark_report(&bismark_file)
            .with_context(|| format!("Failed to parse bismark file for sample: {}", sid))?),
        false => {
            eprintln!("Warning: Bismark file not found: {}", bismark_file);
            None
        }
    };

    // Parse qualimap report (optional)
    let qualimap_file = format!("{}/qualimap/{}_{}", config.qcDir, sid, graft);
    let qualimap_results_file = format!("{}/genome_results.txt", qualimap_file);
    let qualimap_stats = match std::path::Path::new(&qualimap_results_file).exists() {
        true => Some(parse_qualimap_report(&qualimap_results_file)
            .with_context(|| format!("Failed to parse qualimap file for sample: {}", sid))?),
        false => {
            eprintln!("Warning: Qualimap file not found: {}", qualimap_results_file);
            None
        }
    };

    Ok(QCSummary {
        sample_id: sid.to_string(),
        seqkit_stats,
        trim_stats,
        bismark_stats,
        qualimap_stats,
    })
}

pub fn process_sample_rnaseq(config: &QCConfig, sid: &str) -> Result<QCSummaryRNA> {
    // Parse fqc fastqc_data.txt files (replaces seqkit)
    let before = config.qcdir_before.as_deref().unwrap_or(&config.qcDir);
    let after  = config.qcdir_after.as_deref().unwrap_or(&config.qcDir);
    let raw_r1   = format!("{}/{}_R1_fqc/fastqc_data.txt", before, sid);
    let raw_r2   = format!("{}/{}_R2_fqc/fastqc_data.txt", before, sid);
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
    let star_file = format!("{}/{}/{}Log.final.out", config.bsmap_dir, graft, sid);
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
        match process_sample(config, sid) {
            Ok(summary) => summaries.push(summary),
            Err(e) => {
                eprintln!("Warning: Failed to process sample '{}': {}", sid, e);
                // Continue with other samples
            }
        }
    }

    Ok(summaries)
}

pub fn process_all_samples_rnaseq(config: &QCConfig) -> Result<Vec<QCSummaryRNA>> {
    let mut summaries = Vec::new();

    for sid in &config.SIDs {
        match process_sample_rnaseq(config, sid) {
            Ok(summary) => summaries.push(summary),
            Err(e) => {
                eprintln!("Warning: Failed to process sample '{}': {}", sid, e);
                // Continue with other samples
            }
        }
    }

    Ok(summaries)
}
