# Changelog

Generated from this fork's commits after upstream `fb8af35`.
Do not edit by hand. See [changelog maintenance](docs/changelog.md).

## 10.0.0

### Breaking changes

- (rules) add per-rule suppression and selection (`fa85ada`)
  Breaking change: A marker whose brackets list rule ids no longer
  suppresses its function: the function is scored and reported, and only
  the named rules' plans are removed. Core's run_analysis_shared,
  file_complexity_shared, code_complexity_shared and
  function_level_cognitive_complexity_shared take &AnalysisOptions instead
  of separate check_script and no_ignore flags.

### Fixes

- (cli) slice caret spans on character boundaries (`f801094`)

### Refactoring

- (rust) apply machine-applicable pedantic fixes (`dfb0002`)

- (rust) clear remaining pedantic findings (`ebb0ec7`)

- (core) import directly instead of through re-export shims (`5c4628f`)

### Documentation

- (agents) move fork work to rcq and keep main as the upstream mirror (`953455b`)

- (maintenance) refresh claims overtaken by the 9.0.0 release (`0c3123c`)

- (maintenance) record the parent's 9.0.0 adoption (`cd3ac55`)

- (agents) keep main as the GitHub default branch (`2b7e0fe`)

- (agents) track the latest stable Rust (`1821444`)

### Maintenance

- (maintenance) record upstream main fb8af35 as merged (`e3d31a8`)

- (changelog) move the fork baseline to upstream fb8af35 (`8d2f474`)

- (rust) take PYO3_PYTHON from the project venv (`c4333f8`)

- (rust) adopt resolver 3 (`ac7c92a`)

- (rust) lint the feature-isolated CLI build (`18985d2`)

- resolve maturin builds with --locked (`a800abe`)

- (deps) drop the unused globset dependency (`4c6819c`)

- (rust) lock in zero-hit invariants (`02b5161`)

- (rust) require reasoned lint expectations (`0d68f2a`)

- (rust) confine terminal output to its owners (`1497060`)

- (rust) lint string slicing (`e341a88`)

- (clippy) enable the pedantic group (`ffd0387`)

- (rust) lint unnameable types and elided lifetimes (`0fa5f58`)

- (clippy) cap block nesting at today's depth (`a941654`)

- (clippy) ban reading the working directory outside entry points (`1b5c251`)

- (clippy) lint unwrap outside tests (`ed6899b`)
## 9.0.0

### Breaking changes

- (paths) unify analysis and collector root semantics (`13a779b`)
  Breaking change: Relative inputs resolve from base_path or invocation_path,
  which must name an existing directory. In-root results are root-relative;
  outside results are canonical absolute paths, and failures are absolute.
  Use file_complexity(path, base_path=root) for repository-relative identity.
  Python path arguments must be strings, not pathlib.Path objects. Update
  outside-root snapshot keys deliberately when adopting the new behavior.
  Rust runner::collect_file_ignored_locations takes &Path for its base.

- fail closed on incomplete populations and bad config (`00eaec3`)
  Breaking change: Failed-path entries are plain absolute paths without
  appended error text. A malformed or unreadable discovered
  complexipy.toml, .complexipy.toml or pyproject.toml now fails the run
  instead of falling through to the next candidate, and a configuration
  that resolves to no paths is an error. Quiet runs count removable-marker
  collection failures in the exit status.

- (types) expose the shared enums as real Python enums (`db9fb92`)
  Breaking change: The enums are enum.Enum classes. Members gain .name,
  .value and iteration, calling a class with a member's value returns
  that member instead of raising TypeError, and repr() now shows the
  value. Code that recovered names through dir() keeps working but can
  read .name directly.

- (lsp) add the language server on the shared config loader (`da5285f`)
  Breaking change: `complexipy lsp` now starts the language server;
  analyze a path named lsp with `complexipy -- lsp`. Schema errors in
  complexipy.toml or .complexipy.toml report `Invalid config in <path>`
  with the key instead of `Failed to parse <path>` with a line and
  column; the run still fails.

### Fixes

- (maintenance) align local build and verification contracts (`9138278`)

### Documentation

- (maintenance) document current fork and parent work (`458ae1d`)

- (rules) define applicability tier policy (`8ae388f`)

### Tests

- (core) pin rule applicability tiers (`f60594e`)

### Maintenance

- (maintenance) record upstream main 5c52836 as merged (`c07ae49`)

- (changelog) move the fork baseline to upstream 5c52836 (`6abea19`)
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
