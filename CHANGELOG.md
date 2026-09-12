# Changelog

Generated from this fork's commits after upstream `030e207`.
Do not edit by hand. See [changelog maintenance](docs/changelog.md).

## 8.1.0

### Breaking changes

- (rules) remove doc_url and references from RefactorPlan (`39e1bd5`)
  Breaking change: refactor_plans[] entries in --output-format json no
  longer carry doc_url or references, the same two attributes are gone
  from the Python RefactorPlan, SARIF emits neither helpUri nor
  informationUri, and --suggest-refactors no longer prints a References:
  block. rule_id still ships everywhere, so a consumer that wants
  documentation can map the id to a local page itself.

- (fork) align identity and Python 3.14 contracts (`b052287`)
  Breaking change: Requires Python 3.14 or later. Type checking now rejects
  direct construction and attribute assignment for native analysis results,
  matching operations the runtime already rejected.

### Fixes

- (refactors) refuse loop guards when statements follow the chain (`87ad610`)

- (runner) report per-file collector failures (`0f691e7`)

- (stubs) make DiffEntry truthful and drop phantom native helpers (`8a35091`)

- (paths) resolve a relative --output against the invocation path (`b69915b`)

- (python) default compute_diff's invocation_path to the working directory (`f207fb4`)

- (cli) correct six --help strings that described the wrong behavior (`26e8403`)

### Refactoring

- (crates) remove the wasm target and collapse core's features (`48d20d4`)

### Documentation

- (python) clarify result paths and module complexity totals (`fa174cc`)

- (usage-guide) correct the DiffStatus comparison contract (`d690c9f`)

- (realignment) add the fork realignment tracker (`2f48af1`)

- (realignment) settle removals and scope their replacements (`e6ce4c0`)

- (realignment) record the explore sweep and absorb its corrections (`f1b642e`)

- (realignment) record pre-existing defects and sequence them (`0dcfdf9`)

- (realignment) collapse the runner feature in workstream A (`d21c0e1`)

- (realignment) act on the remaining workstream B findings (`634c39a`)

- replace the published site with a local reference (`2020084`)

- (realignment) add the design-issues-and-bugs catalog (`bdb9380`)

- (realignment) reconcile the four documents with the tree before E (`d14c3f3`)

- audit workstream C's pages against the source and pin their claims (`7025244`)

- (skills) replace upstream workflows with local procedures (`e1a17fb`)

### Maintenance

- (release) mark the vendored branch as 8.0.0+rcq.1 (`fac2bd6`)

- (version) package stub corrections as 8.0.0+rcq.2 (`c613bdb`)

- tighten lint and compile checks (`f6d4a14`)

- integrate main and rcq work (`9926391`)

- enforce typing policy without project installs (`5999826`)

- (uv) key the extension cache on Cargo.lock and crate manifests (`6839926`)

- remove CI, contributor machinery, benchmarks and the pi harness (`a5bc269`)

- (tooling) remove the pre-commit stack (`5ec3c37`)

- (changelog) keep fork generation local and simple (`127cb7d`)

- (maintenance) close fork realignment (`ae49dff`)
