use super::record_ignore_error;
use ignore::Error;
use tempfile::tempdir;

#[test]
fn wrapped_pathless_errors_use_the_walk_root() {
    let dir = tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let named = root.join(".ignore");
    let error = Error::Partial(vec![
        Error::WithDepth {
            depth: 2,
            err: Box::new(Error::WithLineNumber {
                line: 3,
                err: Box::new(Error::Io(std::io::Error::other("unreadable"))),
            }),
        },
        Error::WithPath {
            path: named.clone(),
            err: Box::new(Error::Io(std::io::Error::other("unreadable"))),
        },
    ]);
    let mut failed = Vec::new();
    record_ignore_error(&error, &root, &mut failed);
    assert_eq!(failed, [root.to_str().unwrap(), named.to_str().unwrap()]);
}
