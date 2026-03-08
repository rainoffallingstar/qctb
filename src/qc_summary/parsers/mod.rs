pub mod bismark;
pub mod fqc;
pub mod qualimap;
pub mod rnaseq;
pub mod seqkit;
pub mod trim_galore;

pub use bismark::{parse_bismark_report, BismarkStats};
pub use fqc::{parse_fqc_data, parse_seqkit_from_fqc, FqcRow};
pub use qualimap::{parse_qualimap_report, QualimapStats};
pub use rnaseq::{parse_star_log, StarStats};
pub use seqkit::{parse_seqkit, SeqkitStats};
pub use trim_galore::{parse_trim_report, parse_trim_reports, TrimStats};

#[derive(Debug, Clone)]
pub struct QCStats {
    pub sample_id: String,
    pub seqkit_stats: SeqkitStats,
    pub trim_stats: TrimStats,
}
