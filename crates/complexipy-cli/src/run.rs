#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "the CLI owns terminal output"
)]

use std::process::ExitCode;

use owo_colors::OwoColorize;

use crate::args::CliArgs;
use crate::output::messages::{
    diff_flags_warning, handle_snapshot_console, ignored_saved_output, ignored_summary_output,
    removable_ignores_output, unknown_rule_warning,
};
use crate::output::render::{handle_console_settings, print_invalid_paths, rule};
use crate::output::{DisplayOptions, StorageOptions, handle_display, handle_results_storage};
use crate::types::ExitReport;
use crate::utils::config::resolve_config;
use crate::utils::ignored::{handle_removable_ignores, handle_report_ignored};
use crate::utils::snapshot::{SnapshotEvaluation, evaluate_snapshot};
use crate::utils::toml::get_complexipy_toml_config;
use complexipy_core::diff::{
    compute_diff, compute_staged_diff, has_regressions, resolve_diff_flags,
};
use complexipy_core::runner::run_analysis_shared;
use complexipy_core::{AnalysisOptions, RuleSet, registered_rule_ids};

pub fn run_at(cli: CliArgs, invocation_path: &str) -> ExitCode {
    let toml_config = match get_complexipy_toml_config(invocation_path) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    let config = match resolve_config(toml_config.as_ref(), cli) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    let settings = handle_console_settings(&config.color, config.quiet, config.plain);
    if !settings.banner.is_empty() {
        println!("{}", settings.banner);
    }

    let (diff, diff_only) = resolve_diff_flags(config.diff, config.diff_only, config.staged);
    if !config.quiet && diff_only.is_some() && diff.is_none() {
        println!("{} {}", "Warning:".yellow(), diff_flags_warning());
    }

    let (rules, unknown_rules) =
        RuleSet::resolve(&config.select, &config.ignore, &registered_rule_ids());
    if !config.quiet {
        for rule_id in &unknown_rules {
            eprintln!("{} {}", "Warning:".yellow(), unknown_rule_warning(rule_id));
        }
    }
    let analysis = AnalysisOptions {
        check_script: config.check_script,
        no_ignore: config.no_ignore,
        with_plans: true,
        rules,
    };

    let (files_complexities, mut failed_paths) =
        match run_analysis_shared(&config.paths, &config.exclude, &analysis, invocation_path) {
            Ok(result) => result,
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::FAILURE;
            }
        };

    let ignored_report = match handle_report_ignored(
        config.report_ignored,
        &config.paths,
        &config.exclude,
        &config.output_format,
        config.output.as_deref(),
        invocation_path,
    ) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let (removable, removable_failed) = match handle_removable_ignores(
        &config.paths,
        &config.exclude,
        config.max_complexity_allowed,
        invocation_path,
    ) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    failed_paths.extend(ignored_report.failed_paths.iter().cloned());
    failed_paths.extend(removable_failed);
    failed_paths.sort();
    failed_paths.dedup();
    let (paths_ok, invalid_paths_output) = print_invalid_paths(&failed_paths);
    if !invalid_paths_output.is_empty() {
        eprintln!("{invalid_paths_output}");
        eprintln!("Incomplete collection; snapshot and previous-function cache updates skipped.");
    }

    let output_snapshot_path = format!("{invocation_path}/complexipy-snapshot.json");
    let snap = if paths_ok {
        match evaluate_snapshot(
            config.snapshot_create,
            config.snapshot_ignore,
            &output_snapshot_path,
            config.max_complexity_allowed,
            &files_complexities,
        ) {
            Ok(snap) => snap,
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        SnapshotEvaluation {
            should_run: false,
            active_snapshot_map: None,
            watermark_success: true,
            watermark_messages: Vec::new(),
            snapshot_result: true,
        }
    };

    match handle_results_storage(StorageOptions {
        output_formats: &config.output_format,
        output: config.output.as_deref(),
        files_complexities: &files_complexities,
        sort: config.sort.clone(),
        show_details: !config.failed,
        max_complexity: config.max_complexity_allowed,
        invocation_path,
        suggest_refactors: config.suggest_refactors,
    }) {
        Ok(saved_lines) => {
            for line in saved_lines {
                println!("{line}");
            }
        }
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    }

    let (display_ok, display_output) = handle_display(DisplayOptions {
        files_complexities: &files_complexities,
        population_complete: paths_ok,
        paths: &config.paths,
        failed: config.failed,
        sort: config.sort.clone(),
        ignore_complexity: config.ignore_complexity,
        max_complexity_allowed: config.max_complexity_allowed,
        active_snapshot_map: snap.active_snapshot_map.as_ref(),
        quiet: config.quiet,
        plain: config.plain,
        invocation_path,
        cache_dir: config.cache_dir.as_deref(),
        top: config.top,
        suggest_refactors: config.suggest_refactors,
    });
    if !display_output.is_empty() {
        println!("{display_output}");
    }

    if config.report_ignored {
        if !ignored_report.failed_paths.is_empty() {
            eprintln!("Ignore-marker collection incomplete.");
            if config
                .output_format
                .contains(&crate::types::OutputFormat::Json)
            {
                eprintln!("No complete ignored-marker JSON report was written.");
            }
        } else if !config.quiet {
            println!(
                "{}",
                ignored_summary_output(ignored_report.locations.len(), config.no_ignore)
            );
        }
        if let Some(path) = ignored_report.json_path {
            println!("{}", ignored_saved_output(&path));
        }
    }

    if !config.quiet {
        let removable_output = removable_ignores_output(&removable);
        if !removable_output.is_empty() {
            println!("{removable_output}");
        }
    }

    let snapshot_ok = snap.snapshot_result;
    if !config.quiet {
        let snapshot_output = handle_snapshot_console(&snap, &output_snapshot_path);
        if !snapshot_output.is_empty() {
            println!("{snapshot_output}");
        }
    }

    let diff_ref = diff.clone().or_else(|| diff_only.clone());
    let diff_entries = if let Some(diff_ref) = diff_ref {
        if config.staged {
            if let Some(entries) = compute_staged_diff(&diff_ref, invocation_path) {
                if !config.quiet {
                    println!(
                        "{}",
                        format_diff_for(&entries, &format!("{diff_ref} (staged)"))
                    );
                }
                Some(entries)
            } else {
                if !config.quiet {
                    println!(
                        "{} --staged requires a git repository; skipping the staged diff.",
                        "Warning:".yellow()
                    );
                }
                None
            }
        } else if !files_complexities.is_empty() {
            let entries = compute_diff(&files_complexities, &diff_ref, invocation_path);
            if !config.quiet {
                println!("{}", format_diff_for(&entries, &diff_ref));
            }
            Some(entries)
        } else {
            None
        }
    } else {
        None
    };

    let mut diff_ok = true;
    if diff.is_some()
        && let Some(entries) = diff_entries
    {
        diff_ok = !has_regressions(&entries, config.max_complexity_allowed);
    }

    if !config.quiet && !config.plain {
        if cfg!(windows) {
            println!("{}", rule("Analysis completed!"));
        } else {
            println!("{}", rule("🎉 Analysis completed! 🎉"));
        }
    }

    let report = ExitReport {
        display_ok,
        snapshot_ok,
        paths_ok,
        diff_ok,
        enforce_diff: diff.is_some(),
    };
    if report.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn format_diff_for(entries: &[complexipy_core::diff::DiffEntry], git_ref: &str) -> String {
    crate::output::diff::format_diff(entries, git_ref)
}

#[cfg(test)]
mod tests;
