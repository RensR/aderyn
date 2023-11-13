use std::io::{Result, Write};

use crate::{ast::SourceUnit, context::loader::ContextLoader};

use super::reporter::{Issue, Report};

pub trait ReportPrinter {
    fn print_title_and_disclaimer<W: Write>(&self, writer: W) -> Result<()>;
    fn print_report<W: Write>(
        &self,
        writer: W,
        report: &Report,
        loader: &ContextLoader,
    ) -> Result<()>;
    fn print_table_of_contents<W: Write>(&self, writer: W, report: &Report) -> Result<()>;
    fn print_contract_summary<W: Write>(
        &self,
        writer: W,
        report: &Report,
        loader: &ContextLoader,
    ) -> Result<()>;
    fn print_issue<W: Write>(
        &self,
        writer: W,
        issue: &Issue,
        loader: &ContextLoader,
        severity: &str,
        number: i32,
    ) -> Result<()>;
}

pub struct MarkdownReportPrinter;

impl ReportPrinter for MarkdownReportPrinter {
    fn print_report<W: Write>(
        &self,
        mut writer: W,
        report: &Report,
        loader: &ContextLoader,
    ) -> Result<()> {
        self.print_title_and_disclaimer(&mut writer)?;
        self.print_table_of_contents(&mut writer, report)?;
        self.print_contract_summary(&mut writer, report, loader)?;
        let mut counter = 0;
        if !report.criticals.is_empty() {
            writeln!(writer, "# Critical Issues\n")?;
            for issue in &report.criticals {
                counter += 1;
                self.print_issue(&mut writer, issue, loader, "C", counter)?;
            }
        }
        if !report.highs.is_empty() {
            writeln!(writer, "# High Issues\n")?;
            counter = 0;
            for issue in &report.highs {
                counter += 1;
                self.print_issue(&mut writer, issue, loader, "H", counter)?;
            }
        }
        if !report.mediums.is_empty() {
            writeln!(writer, "# Medium Issues\n")?;
            counter = 0;
            for issue in &report.mediums {
                counter += 1;
                self.print_issue(&mut writer, issue, loader, "M", counter)?;
            }
        }
        if !report.lows.is_empty() {
            writeln!(writer, "# Low Issues\n")?;
            counter = 0;
            for issue in &report.lows {
                counter += 1;
                self.print_issue(&mut writer, issue, loader, "L", counter)?;
            }
        }
        if !report.ncs.is_empty() {
            writeln!(writer, "# NC Issues\n")?;
            counter = 0;
            for issue in &report.ncs {
                counter += 1;
                self.print_issue(&mut writer, issue, loader, "NC", counter)?;
            }
        }
        Ok(())
    }

    fn print_title_and_disclaimer<W: Write>(&self, mut writer: W) -> Result<()> {
        writeln!(writer, "# Aderyn Analysis Report\n")?;
        writeln!(
            writer,
            "This report was generated by [Aderyn](https://github.com/Cyfrin/aderyn), a static analysis tool \
            built by [Cyfrin](https://cyfrin.io), a blockchain security company. This report is not a substitute for manual audit or security review. \
            It should not be relied upon for any purpose other than to assist in the identification of potential security vulnerabilities."
        )?;
        Ok(())
    }
    fn print_contract_summary<W: Write>(
        &self,
        mut writer: W,
        report: &Report,
        loader: &ContextLoader,
    ) -> Result<()> {
        writeln!(writer, "# Summary\n")?;

        // Files Summary
        writeln!(writer, "## Files Summary\n")?;
        let total_source_units = loader.get_source_units().len();
        let total_sloc = loader.get_sloc_stats().code;

        // Start the markdown table
        writeln!(writer, "| Key | Value |")?;
        writeln!(writer, "| --- | --- |")?;
        writeln!(writer, "| .sol Files | {} |", total_source_units)?;
        writeln!(writer, "| Total nSLOC | {} |", total_sloc)?;

        writeln!(writer, "\n")?; // Add an extra newline for spacing

        // Files Details
        writeln!(writer, "## Files Details\n")?;

        // Start the markdown table with the header
        writeln!(writer, "| Filepath | nSLOC |")?;
        writeln!(writer, "| --- | --- |")?;

        let sloc_stats = loader.get_sloc_stats();

        // Iterate over source units and add each as a row in the markdown table
        for source_unit in loader.get_source_units() {
            let filepath = source_unit.absolute_path.as_ref().unwrap();
            let report: &tokei::Report = sloc_stats
                .reports
                .iter()
                .find(|r| r.name.to_str().map_or(false, |s| s.contains(filepath)))
                .unwrap();
            writeln!(writer, "| {} | {} |", filepath, report.stats.code)?;
        }
        writeln!(writer, "| **Total** | **{}** |", sloc_stats.code)?;
        writeln!(writer, "\n")?; // Add an extra newline for spacing

        // Analysis Sumarry
        writeln!(writer, "## Issue Summary\n")?;
        // Start the markdown table
        writeln!(writer, "| Category | No. of Issues |")?;
        writeln!(writer, "| --- | --- |")?;
        writeln!(writer, "| Critical | {} |", report.criticals.len())?;
        writeln!(writer, "| High | {} |", report.highs.len())?;
        writeln!(writer, "| Medium | {} |", report.mediums.len())?;
        writeln!(writer, "| Low | {} |", report.lows.len())?;
        writeln!(writer, "| NC | {} |", report.ncs.len())?;
        writeln!(writer, "\n")?; // Add an extra newline for spacing

        Ok(())
    }

    fn print_table_of_contents<W: Write>(&self, mut writer: W, report: &Report) -> Result<()> {
        writeln!(writer, "# Table of Contents\n")?;
        writeln!(writer, "- [Summary](#summary)")?;
        writeln!(writer, "  - [Files Summary](#files-summary)")?;
        writeln!(writer, "  - [Files Details](#files-details)")?;
        writeln!(writer, "  - [Issue Summary](#issue-summary)")?;
        if !report.criticals.is_empty() {
            writeln!(writer, "- [Critical Issues](#critical-issues)")?;
            for (index, issue) in report.criticals.iter().enumerate() {
                let issue_title_slug = issue
                    .title
                    .to_lowercase()
                    .replace(" ", "-")
                    .replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', "");
                writeln!(
                    writer,
                    "  - [C-{}: {}](#C-{}-{})",
                    index + 1,
                    issue.title,
                    index + 1,
                    issue_title_slug
                )?;
            }
        }
        if !report.highs.is_empty() {
            writeln!(writer, "- [High Issues](#high-issues)")?;
            for (index, issue) in report.highs.iter().enumerate() {
                let issue_title_slug = issue
                    .title
                    .to_lowercase()
                    .replace(" ", "-")
                    .replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', "");
                writeln!(
                    writer,
                    "  - [H-{}: {}](#H-{}-{})",
                    index + 1,
                    issue.title,
                    index + 1,
                    issue_title_slug
                )?;
            }
        }
        if !report.mediums.is_empty() {
            writeln!(writer, "- [Medium Issues](#medium-issues)")?;
            for (index, issue) in report.mediums.iter().enumerate() {
                let issue_title_slug = issue
                    .title
                    .to_lowercase()
                    .replace(" ", "-")
                    .replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', "");
                writeln!(
                    writer,
                    "  - [M-{}: {}](#M-{}-{})",
                    index + 1,
                    issue.title,
                    index + 1,
                    issue_title_slug
                )?;
            }
        }
        if !report.lows.is_empty() {
            writeln!(writer, "- [Low Issues](#low-issues)")?;
            for (index, issue) in report.lows.iter().enumerate() {
                let issue_title_slug = issue
                    .title
                    .to_lowercase()
                    .replace(" ", "-")
                    .replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', "");
                writeln!(
                    writer,
                    "  - [L-{}: {}](#L-{}-{})",
                    index + 1,
                    issue.title,
                    index + 1,
                    issue_title_slug
                )?;
            }
        }
        if !report.ncs.is_empty() {
            writeln!(writer, "- [NC Issues](#nc-issues)")?;
            for (index, issue) in report.ncs.iter().enumerate() {
                let issue_title_slug = issue
                    .title
                    .to_lowercase()
                    .replace(" ", "-")
                    .replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', "");
                writeln!(
                    writer,
                    "  - [NC-{}: {}](#NC-{}-{})",
                    index + 1,
                    issue.title,
                    index + 1,
                    issue_title_slug
                )?;
            }
        }
        writeln!(writer, "\n")?; // Add an extra newline for spacing
        Ok(())
    }

    fn print_issue<W: Write>(
        &self,
        mut writer: W,
        issue: &Issue,
        loader: &ContextLoader,
        severity: &str,
        number: i32,
    ) -> Result<()> {
        writeln!(
            writer,
            "## {}-{}: {}\n\n{}\n", // <a name> is the anchor for the issue title
            severity, number, issue.title, issue.description
        )?;
        for node in issue.instances.iter().flatten() {
            let mut contract_path = "unknown";
            let source_unit: &SourceUnit = loader.get_source_unit_from_child_node(node).unwrap();
            if let Some(path) = source_unit.absolute_path.as_ref() {
                contract_path = path;
            }
            let mut line_number = 0;
            if let Some(src) = node.src() {
                line_number = source_unit.source_line(src).unwrap();
            }
            writeln!(
                writer,
                "- Found in {}: Line: {}",
                contract_path, line_number
            )?;
        }
        writeln!(writer, "\n")?; // Add an extra newline for spacing
        Ok(())
    }
}
