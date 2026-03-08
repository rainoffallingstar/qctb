use crate::qc_summary::{QCSummary, QCSummaryRNA};
use anyhow::Result;
use rust_xlsxwriter::*;

pub fn write_excel_standard(summaries: &[QCSummary], output_path: &str) -> Result<()> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();

    // Define formats
    let header_format = Format::new()
        .set_bold()
        .set_font_color(Color::RGB(0xFFFFFF))
        .set_background_color(Color::RGB(0x4F81BD))
        .set_align(FormatAlign::Center);

    let cell_format = Format::new().set_align(FormatAlign::Left);

    // Write headers
    let headers = vec![
        "Sample ID",
        "Reads Raw",
        "Bases Raw",
        "Reads Clean",
        "Bases Clean",
        "Clean Data Ratio",
        "Q20 Raw R1 (%)",
        "Q30 Raw R1 (%)",
        "Avg Len Raw R1",
        "Q20 Raw R2 (%)",
        "Q30 Raw R2 (%)",
        "Avg Len Raw R2",
        "Q20 Clean R1 (%)",
        "Q30 Clean R1 (%)",
        "Avg Len Clean R1",
        "Q20 Clean R2 (%)",
        "Q30 Clean R2 (%)",
        "Avg Len Clean R2",
        "Mapping Ratio (%)",
        "Total Read Pairs",
        "Aligned Read Pairs",
        "Aligned Ratio",
        "Mapping Quality",
        "Duplicated Reads",
        "Duplication Rate",
    ];

    for (col, header) in headers.iter().enumerate() {
        sheet.write_string_with_format(0, col as u16, *header, &header_format)?;
    }

    // Write data
    for (row, summary) in summaries.iter().enumerate() {
        let row_num = (row + 1) as u32;
        let s = &summary.seqkit_stats;

        sheet.write_string_with_format(row_num, 0, summary.sample_id.as_str(), &cell_format)?;
        sheet.write_number_with_format(row_num, 1, s.reads_raw as f64, &cell_format)?;
        sheet.write_number_with_format(row_num, 2, s.bases_raw as f64, &cell_format)?;
        sheet.write_number_with_format(row_num, 3, s.reads_clean as f64, &cell_format)?;
        sheet.write_number_with_format(row_num, 4, s.bases_clean as f64, &cell_format)?;
        sheet.write_number_with_format(row_num, 5, s.clean_data_ratio, &cell_format)?;
        sheet.write_number_with_format(row_num, 6, s.q20_raw_r1, &cell_format)?;
        sheet.write_number_with_format(row_num, 7, s.q30_raw_r1, &cell_format)?;
        sheet.write_number_with_format(row_num, 8, s.avg_len_raw_r1, &cell_format)?;
        sheet.write_number_with_format(row_num, 9, s.q20_raw_r2, &cell_format)?;
        sheet.write_number_with_format(row_num, 10, s.q30_raw_r2, &cell_format)?;
        sheet.write_number_with_format(row_num, 11, s.avg_len_raw_r2, &cell_format)?;
        sheet.write_number_with_format(row_num, 12, s.q20_clean_r1, &cell_format)?;
        sheet.write_number_with_format(row_num, 13, s.q30_clean_r1, &cell_format)?;
        sheet.write_number_with_format(row_num, 14, s.avg_len_clean_r1, &cell_format)?;
        sheet.write_number_with_format(row_num, 15, s.q20_clean_r2, &cell_format)?;
        sheet.write_number_with_format(row_num, 16, s.q30_clean_r2, &cell_format)?;
        sheet.write_number_with_format(row_num, 17, s.avg_len_clean_r2, &cell_format)?;

        // Bismark stats (optional)
        if let Some(ref bs) = summary.bismark_stats {
            sheet.write_string_with_format(row_num, 18, bs.mapping_ratio.as_str(), &cell_format)?;
            sheet.write_string_with_format(
                row_num,
                19,
                bs.total_reads_pairs.as_str(),
                &cell_format,
            )?;
            sheet.write_string_with_format(
                row_num,
                20,
                bs.aligned_reads_pairs.as_str(),
                &cell_format,
            )?;
            sheet.write_number_with_format(
                row_num,
                21,
                bs.aligned_reads_pairs_ratio,
                &cell_format,
            )?;
        } else {
            for col in 18..=21 {
                sheet.write_string_with_format(row_num, col, "N/A", &cell_format)?;
            }
        }

        // Qualimap stats (optional)
        if let Some(ref qs) = summary.qualimap_stats {
            sheet.write_string_with_format(
                row_num,
                22,
                qs.mapping_quality.as_str(),
                &cell_format,
            )?;
            sheet.write_string_with_format(
                row_num,
                23,
                qs.duplicated_reads.as_str(),
                &cell_format,
            )?;
            sheet.write_string_with_format(
                row_num,
                24,
                qs.duplication_ratio.as_str(),
                &cell_format,
            )?;
        } else {
            for col in 22..=24 {
                sheet.write_string_with_format(row_num, col, "N/A", &cell_format)?;
            }
        }
    }

    // Auto-fit columns
    for col in 0..25 {
        sheet.set_column_width(col as u16, 18)?;
    }

    workbook.save(output_path)?;
    Ok(())
}

pub fn write_excel_rnaseq(summaries: &[QCSummaryRNA], output_path: &str) -> Result<()> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();

    // Define formats
    let header_format = Format::new()
        .set_bold()
        .set_font_color(Color::RGB(0xFFFFFF))
        .set_background_color(Color::RGB(0x4F81BD))
        .set_align(FormatAlign::Center);

    let cell_format = Format::new().set_align(FormatAlign::Left);

    // Write headers
    let headers = vec![
        "Sample ID",
        "Reads Raw",
        "Bases Raw",
        "Reads Clean",
        "Bases Clean",
        "Clean Data Ratio",
        "Q20 Raw R1 (%)",
        "Q30 Raw R1 (%)",
        "Avg Len Raw R1",
        "Q20 Raw R2 (%)",
        "Q30 Raw R2 (%)",
        "Avg Len Raw R2",
        "Q20 Clean R1 (%)",
        "Q30 Clean R1 (%)",
        "Avg Len Clean R1",
        "Q20 Clean R2 (%)",
        "Q30 Clean R2 (%)",
        "Avg Len Clean R2",
        "Mapping Ratio (%)",
        "Total Reads",
        "Uniquely Mapped Reads",
        "Uniquely Mapped Ratio",
    ];

    for (col, header) in headers.iter().enumerate() {
        sheet.write_string_with_format(0, col as u16, *header, &header_format)?;
    }

    // Write data
    for (row, summary) in summaries.iter().enumerate() {
        let row_num = (row + 1) as u32;
        let s = &summary.seqkit_stats;
        let st = &summary.star_stats;

        sheet.write_string_with_format(row_num, 0, summary.sample_id.as_str(), &cell_format)?;
        sheet.write_number_with_format(row_num, 1, s.reads_raw as f64, &cell_format)?;
        sheet.write_number_with_format(row_num, 2, s.bases_raw as f64, &cell_format)?;
        sheet.write_number_with_format(row_num, 3, s.reads_clean as f64, &cell_format)?;
        sheet.write_number_with_format(row_num, 4, s.bases_clean as f64, &cell_format)?;
        sheet.write_number_with_format(row_num, 5, s.clean_data_ratio, &cell_format)?;
        sheet.write_number_with_format(row_num, 6, s.q20_raw_r1, &cell_format)?;
        sheet.write_number_with_format(row_num, 7, s.q30_raw_r1, &cell_format)?;
        sheet.write_number_with_format(row_num, 8, s.avg_len_raw_r1, &cell_format)?;
        sheet.write_number_with_format(row_num, 9, s.q20_raw_r2, &cell_format)?;
        sheet.write_number_with_format(row_num, 10, s.q30_raw_r2, &cell_format)?;
        sheet.write_number_with_format(row_num, 11, s.avg_len_raw_r2, &cell_format)?;
        sheet.write_number_with_format(row_num, 12, s.q20_clean_r1, &cell_format)?;
        sheet.write_number_with_format(row_num, 13, s.q30_clean_r1, &cell_format)?;
        sheet.write_number_with_format(row_num, 14, s.avg_len_clean_r1, &cell_format)?;
        sheet.write_number_with_format(row_num, 15, s.q20_clean_r2, &cell_format)?;
        sheet.write_number_with_format(row_num, 16, s.q30_clean_r2, &cell_format)?;
        sheet.write_number_with_format(row_num, 17, s.avg_len_clean_r2, &cell_format)?;
        sheet.write_string_with_format(row_num, 18, st.mapping_ratio.as_str(), &cell_format)?;
        sheet.write_string_with_format(row_num, 19, st.total_reads.as_str(), &cell_format)?;
        sheet.write_string_with_format(
            row_num,
            20,
            st.uniquely_mapped_reads.as_str(),
            &cell_format,
        )?;
        sheet.write_number_with_format(row_num, 21, st.uniquely_mapped_ratio, &cell_format)?;
    }

    // Auto-fit columns
    for col in 0..22 {
        sheet.set_column_width(col as u16, 18)?;
    }

    workbook.save(output_path)?;
    Ok(())
}
