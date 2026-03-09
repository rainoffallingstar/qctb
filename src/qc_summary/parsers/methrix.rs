use anyhow::{Context, Result};
use calamine::{open_workbook_auto, Data, Reader};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct MethrixCoverageRow {
    pub sample: String,
    pub total_cpgs: u64,
    pub covered_cpgs: u64,
    pub cov_1x: u64,
    pub cov_2x: u64,
    pub cov_3x: u64,
    pub cov_4x: u64,
    pub cov_5x: u64,
    pub cov_10x: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethrixAnnotationBySampleRow {
    pub sample: String,
    pub covered_cpgs: u64,
    // Dynamic columns from ChIPseeker_By_Sample such as:
    // Promoter_count / Promoter_percent / Exon_count / ...
    pub metrics: BTreeMap<String, f64>,
}

fn cell_as_string(cell: Option<&Data>) -> String {
    match cell {
        Some(v) => v.to_string().trim().to_string(),
        None => String::new(),
    }
}

fn cell_as_u64(cell: Option<&Data>, field: &str) -> Result<u64> {
    let raw = cell_as_string(cell);
    let value: f64 = raw
        .parse()
        .with_context(|| format!("Failed to parse '{}' as number: '{}'", field, raw))?;
    Ok(value.round() as u64)
}

fn cell_as_f64(cell: Option<&Data>, field: &str) -> Result<f64> {
    let raw = cell_as_string(cell);
    raw.parse()
        .with_context(|| format!("Failed to parse '{}' as number: '{}'", field, raw))
}

pub fn parse_methrix_coverage_xlsx(path: &str) -> Result<Vec<MethrixCoverageRow>> {
    let mut workbook =
        open_workbook_auto(path).with_context(|| format!("Failed to open XLSX: {}", path))?;
    let range = workbook
        .worksheet_range("Sheet1")
        .with_context(|| format!("Failed to read Sheet1 in {}", path))?;

    let mut rows = range.rows();
    let header = rows.next().context("Coverage sheet is empty")?;
    let headers: Vec<String> = header.iter().map(|c| c.to_string()).collect();

    let find = |name: &str| -> Result<usize> {
        headers
            .iter()
            .position(|h| h == name)
            .with_context(|| format!("Missing required column '{}'", name))
    };

    let idx_sample = find("Sample")?;
    let idx_total = find("Total CpGs")?;
    let idx_covered = find("Covered CpGs")?;
    let idx_1x = find("1X")?;
    let idx_2x = find("2X")?;
    let idx_3x = find("3X")?;
    let idx_4x = find("4X")?;
    let idx_5x = find("5X")?;
    let idx_10x = find("10X")?;

    let mut out = Vec::new();
    for row in rows {
        if cell_as_string(row.get(idx_sample)).is_empty() {
            continue;
        }
        out.push(MethrixCoverageRow {
            sample: cell_as_string(row.get(idx_sample)),
            total_cpgs: cell_as_u64(row.get(idx_total), "Total CpGs")?,
            covered_cpgs: cell_as_u64(row.get(idx_covered), "Covered CpGs")?,
            cov_1x: cell_as_u64(row.get(idx_1x), "1X")?,
            cov_2x: cell_as_u64(row.get(idx_2x), "2X")?,
            cov_3x: cell_as_u64(row.get(idx_3x), "3X")?,
            cov_4x: cell_as_u64(row.get(idx_4x), "4X")?,
            cov_5x: cell_as_u64(row.get(idx_5x), "5X")?,
            cov_10x: cell_as_u64(row.get(idx_10x), "10X")?,
        });
    }

    Ok(out)
}

pub fn parse_methrix_annotation_by_sample_xlsx(
    path: &str,
) -> Result<Vec<MethrixAnnotationBySampleRow>> {
    let mut workbook =
        open_workbook_auto(path).with_context(|| format!("Failed to open XLSX: {}", path))?;
    let range = workbook
        .worksheet_range("ChIPseeker_By_Sample")
        .with_context(|| format!("Failed to read ChIPseeker_By_Sample in {}", path))?;

    let mut rows = range.rows();
    let header = rows.next().context("Annotation-by-sample sheet is empty")?;
    let headers: Vec<String> = header.iter().map(|c| c.to_string()).collect();

    let idx_sample = headers
        .iter()
        .position(|h| h == "sample")
        .context("Missing 'sample' column in ChIPseeker_By_Sample")?;
    let idx_covered = headers
        .iter()
        .position(|h| h == "covered_cpgs")
        .context("Missing 'covered_cpgs' column in ChIPseeker_By_Sample")?;

    let metric_indices: Vec<(usize, String)> = headers
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != idx_sample && *i != idx_covered)
        .map(|(i, h)| (i, h.clone()))
        .collect();

    let mut out = Vec::new();
    for row in rows {
        if cell_as_string(row.get(idx_sample)).is_empty() {
            continue;
        }
        let mut metrics = BTreeMap::new();
        for (idx, name) in &metric_indices {
            let value = cell_as_f64(row.get(*idx), name)?;
            metrics.insert(name.clone(), value);
        }
        out.push(MethrixAnnotationBySampleRow {
            sample: cell_as_string(row.get(idx_sample)),
            covered_cpgs: cell_as_u64(row.get(idx_covered), "covered_cpgs")?,
            metrics,
        });
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn td() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("testdata")
            .join("methrix-qc-excel")
    }

    #[test]
    fn parse_coverage_report() -> Result<()> {
        let file = td().join("CpG_coverage.xlsx");
        let rows = parse_methrix_coverage_xlsx(file.to_str().unwrap())?;
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].sample, "0108ZYHHPC70311_nsort.bismark.cov");
        assert_eq!(rows[0].total_cpgs, 80028);
        assert_eq!(rows[0].covered_cpgs, 35892);
        assert_eq!(rows[1].sample, "0108ZYHHPC70315_nsort.bismark.cov");
        assert_eq!(rows[1].cov_10x, 46249);
        Ok(())
    }

    #[test]
    fn parse_recomputed_coverage_report() -> Result<()> {
        let file = td().join("CpG_coverage_recomputed_from_h5.xlsx");
        let rows = parse_methrix_coverage_xlsx(file.to_str().unwrap())?;
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].covered_cpgs, 35892);
        assert_eq!(rows[1].covered_cpgs, 46253);
        Ok(())
    }

    #[test]
    fn parse_annotation_by_sample_report() -> Result<()> {
        let file = td().join("CpG_annotation_report.xlsx");
        let rows = parse_methrix_annotation_by_sample_xlsx(file.to_str().unwrap())?;
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].sample, "0108ZYHHPC70311_nsort.bismark.cov");
        assert_eq!(rows[0].covered_cpgs, 35892);
        assert!(rows[0].metrics.contains_key("Promoter_count"));
        assert!(rows[0].metrics.contains_key("Promoter_percent"));
        assert_eq!(
            rows[1]
                .metrics
                .get("Intergenic_count")
                .copied()
                .unwrap()
                .round() as u64,
            11283
        );
        Ok(())
    }
}
