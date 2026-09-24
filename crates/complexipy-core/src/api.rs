use crate::classes::{CodeComplexity, FileComplexity};
use crate::cognitive_complexity::code_complexity_shared;
use crate::rules::{AnalysisOptions, RuleSet};
use crate::runner::file_complexity_shared;

pub fn code_complexity(
    code: &str,
    check_script: bool,
    no_ignore: bool,
) -> Result<CodeComplexity, String> {
    code_complexity_shared(
        code,
        &AnalysisOptions {
            check_script,
            no_ignore,
            with_plans: true,
            rules: RuleSet::default(),
        },
    )
}

pub fn file_complexity(
    file_path: &str,
    check_script: bool,
    no_ignore: bool,
) -> Result<FileComplexity, String> {
    file_complexity_shared(
        file_path,
        ".",
        &AnalysisOptions {
            check_script,
            no_ignore,
            with_plans: true,
            rules: RuleSet::default(),
        },
    )
}

#[cfg(test)]
mod tests;
