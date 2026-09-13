use crate::classes::{CodeComplexity, FileComplexity};
use crate::cognitive_complexity::code_complexity_shared;
use crate::runner::file_complexity_shared;

pub fn code_complexity(
    code: &str,
    check_script: bool,
    no_ignore: bool,
) -> Result<CodeComplexity, String> {
    code_complexity_shared(code, check_script, no_ignore)
}

pub fn file_complexity(
    file_path: &str,
    check_script: bool,
    no_ignore: bool,
) -> Result<FileComplexity, String> {
    file_complexity_shared(file_path, ".", check_script, no_ignore)
}

#[cfg(test)]
mod tests;
