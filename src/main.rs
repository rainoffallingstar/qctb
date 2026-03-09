use anyhow::{Context, Result};
use clap::Parser;
use std::path::Path;

mod qc_summary;

#[derive(Parser)]
#[command(name = "qctb")]
#[command(about = "QC tools for bioinformatics", long_about = None)]
#[command(version)]
struct Cli {
    /// YAML configuration file
    #[arg(long)]
    config: String,

    /// Output file path
    #[arg(long)]
    output: String,

    /// Output format: xlsx or tsv (default: xlsx)
    #[arg(long, default_value = "xlsx")]
    format: String,

    /// RNA-seq mode (use RNA-seq specific metrics and parsers)
    #[arg(long)]
    rnaseq: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load config
    let config_path = Path::new(&cli.config);
    let qc_config = qc_summary::load_config(config_path)
        .with_context(|| format!("Failed to load config from: {}", cli.config))?;

    println!(
        "Processing {} samples in {} mode...",
        qc_config.SIDs.len(),
        if cli.rnaseq { "RNA-seq" } else { "standard" }
    );
    println!("Output format: {}", cli.format);

    if cli.rnaseq {
        // RNA-seq mode
        let summaries = qc_summary::process_all_samples_rnaseq(&qc_config)
            .with_context(|| "Failed to process samples in RNA-seq mode")?;

        println!("Successfully processed {} samples", summaries.len());

        if cli.format == "xlsx" {
            qc_summary::write_excel_rnaseq(&summaries, &cli.output)
                .with_context(|| format!("Failed to write Excel output to: {}", cli.output))?;
        } else {
            let path = Path::new(&cli.output);
            write_tsv_summary_rnaseq(&summaries, path)
                .with_context(|| format!("Failed to write TSV output to: {}", cli.output))?;
        }
    } else {
        // Standard mode
        let summaries = qc_summary::process_all_samples(&qc_config)
            .with_context(|| "Failed to process samples")?;

        println!("Successfully processed {} samples", summaries.len());

        if cli.format == "xlsx" {
            qc_summary::write_excel_standard(&summaries, &cli.output)
                .with_context(|| format!("Failed to write Excel output to: {}", cli.output))?;
        } else {
            let path = Path::new(&cli.output);
            write_tsv_summary(&summaries, path)
                .with_context(|| format!("Failed to write TSV output to: {}", cli.output))?;
        }
    }

    println!("Output written to: {}", cli.output);
    Ok(())
}

fn write_tsv_summary(
    summaries: &[qc_summary::QCSummary],
    output_path: &std::path::Path,
) -> Result<()> {
    use std::io::Write;

    let mut file = std::fs::File::create(output_path)?;
    let header = "sample_id\treads_raw\tbases_raw\treads_clean\tbases_clean\tclean_data_ratio\tq20_raw_r1\tq30_raw_r1\tavg_len_raw_r1\tq20_raw_r2\tq30_raw_r2\tavg_len_raw_r2\tq20_clean_r1\tq30_clean_r1\tavg_len_clean_r1\tq20_clean_r2\tq30_clean_r2\tavg_len_clean_r2\tmapping_ratio\ttotal_reads_pairs\taligned_reads_pairs\taligned_ratio\tmapping_quality\tduplicated_reads\tduplication_ratio\tmethrix_total_cpgs\tmethrix_covered_cpgs\tmethrix_1x\tmethrix_2x\tmethrix_3x\tmethrix_4x\tmethrix_5x\tmethrix_10x\tmethrix_ann_covered_cpgs\tmethrix_promoter_count\tmethrix_promoter_percent\tmethrix_exon_count\tmethrix_exon_percent\tmethrix_intron_count\tmethrix_intron_percent\tmethrix_intergenic_count\tmethrix_intergenic_percent\n";
    file.write_all(header.as_bytes())?;

    for summary in summaries {
        let s = &summary.seqkit_stats;

        // Get optional bismark stats
        let (mapping_ratio, total_pairs, aligned_pairs, aligned_ratio) =
            if let Some(ref bs) = summary.bismark_stats {
                (
                    bs.mapping_ratio.clone(),
                    bs.total_reads_pairs.clone(),
                    bs.aligned_reads_pairs.clone(),
                    format!("{:.4}", bs.aligned_reads_pairs_ratio),
                )
            } else {
                (
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A".to_string(),
                )
            };

        // Get optional qualimap stats
        let (map_quality, dup_reads, dup_ratio) = if let Some(ref qs) = summary.qualimap_stats {
            (
                qs.mapping_quality.clone(),
                qs.duplicated_reads.clone(),
                qs.duplication_ratio.clone(),
            )
        } else {
            ("N/A".to_string(), "N/A".to_string(), "N/A".to_string())
        };

        let (mc_total, mc_covered, mc_1x, mc_2x, mc_3x, mc_4x, mc_5x, mc_10x) =
            if let Some(ref mc) = summary.methrix_coverage {
                (
                    mc.total_cpgs.to_string(),
                    mc.covered_cpgs.to_string(),
                    mc.cov_1x.to_string(),
                    mc.cov_2x.to_string(),
                    mc.cov_3x.to_string(),
                    mc.cov_4x.to_string(),
                    mc.cov_5x.to_string(),
                    mc.cov_10x.to_string(),
                )
            } else {
                (
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A".to_string(),
                )
            };

        let (
            ma_covered,
            ma_promoter_count,
            ma_promoter_pct,
            ma_exon_count,
            ma_exon_pct,
            ma_intron_count,
            ma_intron_pct,
            ma_intergenic_count,
            ma_intergenic_pct,
        ) = if let Some(ref ma) = summary.methrix_annotation {
            let metric = |k: &str| ma.metrics.get(k).copied().unwrap_or(0.0);
            (
                ma.covered_cpgs.to_string(),
                metric("Promoter_count").to_string(),
                metric("Promoter_percent").to_string(),
                metric("Exon_count").to_string(),
                metric("Exon_percent").to_string(),
                metric("Intron_count").to_string(),
                metric("Intron_percent").to_string(),
                metric("Intergenic_count").to_string(),
                metric("Intergenic_percent").to_string(),
            )
        } else {
            (
                "N/A".to_string(),
                "N/A".to_string(),
                "N/A".to_string(),
                "N/A".to_string(),
                "N/A".to_string(),
                "N/A".to_string(),
                "N/A".to_string(),
                "N/A".to_string(),
                "N/A".to_string(),
            )
        };

        let line = format!(
            "{}\t{}\t{}\t{}\t{}\t{:.4}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            summary.sample_id,
            s.reads_raw,
            s.bases_raw,
            s.reads_clean,
            s.bases_clean,
            s.clean_data_ratio,
            s.q20_raw_r1,
            s.q30_raw_r1,
            s.avg_len_raw_r1,
            s.q20_raw_r2,
            s.q30_raw_r2,
            s.avg_len_raw_r2,
            s.q20_clean_r1,
            s.q30_clean_r1,
            s.avg_len_clean_r1,
            s.q20_clean_r2,
            s.q30_clean_r2,
            s.avg_len_clean_r2,
            mapping_ratio,
            total_pairs,
            aligned_pairs,
            aligned_ratio,
            map_quality,
            dup_reads,
            dup_ratio,
            mc_total,
            mc_covered,
            mc_1x,
            mc_2x,
            mc_3x,
            mc_4x,
            mc_5x,
            mc_10x,
            ma_covered,
            ma_promoter_count,
            ma_promoter_pct,
            ma_exon_count,
            ma_exon_pct,
            ma_intron_count,
            ma_intron_pct,
            ma_intergenic_count,
            ma_intergenic_pct,
        );
        file.write_all(line.as_bytes())?;
    }

    Ok(())
}

fn write_tsv_summary_rnaseq(
    summaries: &[qc_summary::QCSummaryRNA],
    output_path: &std::path::Path,
) -> Result<()> {
    use std::io::Write;

    let mut file = std::fs::File::create(output_path)?;
    let header = "sample_id\treads_raw\tbases_raw\treads_clean\tbases_clean\tclean_data_ratio\tq20_raw_r1\tq30_raw_r1\tavg_len_raw_r1\tq20_raw_r2\tq30_raw_r2\tavg_len_raw_r2\tq20_clean_r1\tq30_clean_r1\tavg_len_clean_r1\tq20_clean_r2\tq30_clean_r2\tavg_len_clean_r2\tmapping_ratio\ttotal_reads\tuniquely_mapped_reads\tuniquely_mapped_ratio\n";
    file.write_all(header.as_bytes())?;

    for summary in summaries {
        let s = &summary.seqkit_stats;
        let st = &summary.star_stats;
        let line = format!(
            "{}\t{}\t{}\t{}\t{}\t{:.4}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{}\t{}\t{}\t{:.4}\n",
            summary.sample_id,
            s.reads_raw,
            s.bases_raw,
            s.reads_clean,
            s.bases_clean,
            s.clean_data_ratio,
            s.q20_raw_r1,
            s.q30_raw_r1,
            s.avg_len_raw_r1,
            s.q20_raw_r2,
            s.q30_raw_r2,
            s.avg_len_raw_r2,
            s.q20_clean_r1,
            s.q30_clean_r1,
            s.avg_len_clean_r1,
            s.q20_clean_r2,
            s.q30_clean_r2,
            s.avg_len_clean_r2,
            st.mapping_ratio,
            st.total_reads,
            st.uniquely_mapped_reads,
            st.uniquely_mapped_ratio,
        );
        file.write_all(line.as_bytes())?;
    }

    Ok(())
}
