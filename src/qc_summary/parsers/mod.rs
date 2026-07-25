pub mod bismark;
pub mod fqc;
pub mod methrix;
pub mod qualimap;
pub mod rnaseq;
pub mod stats;
pub mod trim_galore;

pub use bismark::{parse_bismark_report, BismarkStats};
pub use fqc::{parse_fqc_data, parse_seqkit_from_fqc, FqcRow};
pub use methrix::{
    parse_methrix_annotation_by_sample_xlsx, parse_methrix_coverage_xlsx,
    MethrixAnnotationBySampleRow, MethrixCoverageRow,
};
pub use qualimap::{parse_qualimap_report, QualimapStats};
pub use rnaseq::{parse_star_log, StarStats};
pub use stats::SeqkitStats;
pub use trim_galore::{parse_trim_report, parse_trim_reports, TrimStats};
