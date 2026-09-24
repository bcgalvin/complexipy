use clap::Parser;

use crate::types::{Color, OutputFormat, Sort};

#[derive(Parser, Debug, Clone, PartialEq)]
#[command(
    name = "complexipy",
    version,
    about = "Cognitive complexity analysis for Python",
    long_about = "Measures how hard Python code is to understand, after G. Ann \
Campbell's cognitive complexity paper. Analyzes files, directories, or a whole \
repository; gates on a threshold; and compares against a git reference or a \
committed snapshot.\n\nConfiguration is read from the first of complexipy.toml, \
.complexipy.toml, or pyproject.toml [tool.complexipy] found in the invocation \
directory. The first hit wins outright - the files are not merged - and there is \
no upward search."
)]
pub struct CliArgs {
    #[arg(help = "Files or directories to analyze. Defaults to the configured paths")]
    pub paths: Vec<String>,

    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "Glob patterns to skip, comma-separated"
    )]
    pub exclude: Vec<String>,

    #[arg(
        long,
        value_delimiter = ',',
        help = "Report only these refactor rule ids, comma-separated (e.g. C001,C007)"
    )]
    pub select: Vec<String>,

    #[arg(
        long,
        value_delimiter = ',',
        help = "Never report these refactor rule ids, comma-separated; wins over --select"
    )]
    pub ignore: Vec<String>,

    #[arg(long, help = "Fail any function scoring above this")]
    pub max_complexity_allowed: Option<u64>,

    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true, help = "Write current violations to complexipy-snapshot.json in the invocation directory")]
    pub snapshot_create: Option<bool>,

    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true, help = "Skip the snapshot comparison even if the file exists")]
    pub snapshot_ignore: Option<bool>,

    #[arg(short, long, num_args = 0..=1, default_missing_value = "true", require_equals = true, help = "Suppress the results table; print only the summary")]
    pub quiet: Option<bool>,

    #[arg(short, long, num_args = 0..=1, default_missing_value = "true", require_equals = true, help = "Report scores but never fail on them")]
    pub ignore_complexity: Option<bool>,

    #[arg(short, long, num_args = 0..=1, default_missing_value = "true", require_equals = true, help = "Show only functions above the threshold")]
    pub failed: Option<bool>,

    #[arg(
        short = 'C',
        long,
        help = "Color mode. Currently advisory: output is always colored"
    )]
    pub color: Option<Color>,

    #[arg(short, long, help = "Row order: asc, desc, or file_name")]
    pub sort: Option<Sort>,

    #[arg(
        long,
        help = "Destination for machine-readable output. A trailing separator means a directory, which is required for multiple formats. Stdout is not supported"
    )]
    pub output: Option<String>,

    #[arg(
        long,
        help = "Where to keep the analysis cache. Defaults to .complexipy_cache in the invocation directory"
    )]
    pub cache_dir: Option<String>,

    #[arg(
        long,
        value_delimiter = ',',
        help = "Machine-readable formats to write, comma-separated: csv, json, gitlab, sarif"
    )]
    pub output_format: Option<Vec<OutputFormat>>,

    #[arg(
        short,
        long,
        help = "Compare against a git reference. The exit code then gates on regressions above the threshold instead of on the threshold itself"
    )]
    pub diff: Option<String>,

    #[arg(
        long,
        help = "Show the comparison against a git reference without changing the exit code"
    )]
    pub diff_only: Option<String>,

    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true, help = "Diff against the index rather than the working tree")]
    pub staged: Option<bool>,

    #[arg(short, long, value_parser = clap::value_parser!(u64).range(1..), help = "Show only the N most complex functions")]
    pub top: Option<u64>,

    #[arg(long, conflicts_with = "quiet", num_args = 0..=1, default_missing_value = "true", require_equals = true, help = "Plain text output: path, name, and score only")]
    pub plain: Option<bool>,

    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true, help = "Emit refactor plans for every listed function")]
    pub suggest_refactors: Option<bool>,

    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true, help = "Also score module-level code, reported as <module>")]
    pub check_script: Option<bool>,

    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true, help = "Disregard every complexipy ignore comment")]
    pub no_ignore: Option<bool>,

    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true, help = "List every complexipy ignore comment; removable ones are reported on every run")]
    pub report_ignored: Option<bool>,
}

#[cfg(test)]
mod tests;
