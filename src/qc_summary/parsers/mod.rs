pub mod seqkit;
pub mod trim_galore;
pub mod rnaseq;
pub mod bismark;
pub mod qualimap;
pub mod fqc;

pub use seqkit::{SeqkitStats, parse_seqkit};
pub use trim_galore::{TrimStats, parse_trim_report, parse_trim_reports};
pub use rnaseq::{StarStats, parse_star_log};
pub use bismark::{BismarkStats, parse_bismark_report};
pub use qualimap::{QualimapStats, parse_qualimap_report};
pub use fqc::{FqcRow, parse_fqc_data, parse_seqkit_from_fqc};

#[derive(Debug, Clone)]
pub struct QCStats {
    pub sample_id: String,
    pub seqkit_stats: SeqkitStats,
    pub trim_stats: TrimStats,
}
