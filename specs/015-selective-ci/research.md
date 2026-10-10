# Research: Selective CI by Change Impact

## Decisions

### Keep a workflow run on every PR update

GitHub workflow-level path filtering can prevent a check run from being created and leave required checks pending. Job-level conditions allow one current-SHA workflow to report intentional skips. A successful check on a prior commit is not a result for a later SHA. Therefore this feature changes job selection, not workflow triggers.

### Use an exact base-to-merge tree diff

The changed paths must describe the tree tested by the jobs. Use the PR event's base SHA and `github.sha` merge SHA directly rather than a merge-base diff. Request NUL-delimited `git diff --name-status` output with rename/copy detection; process both sides of renames/copies and the old side of deletions. Git invocation uses an argument list and reports malformed output as a conservative full-CI fallback.

### Narrow only reviewed independent component paths

The workflow's adapters share build dependencies and test integration. Only source, test, and example subtrees with a clear owning component are narrow. Workspace/source code, build configuration, manifests/locks, scripts/tool config, generators, normative inputs, and unknown paths select full CI. This keeps false negatives more expensive than the occasional extra job.

### Split combined jobs before routing

The existing language job performs C, Java/JNI, and Python work for each OS; the adapter coverage job produces Java, Python, and JNI reports together. Per-language selection is impossible until those responsibilities are separated. Preserve each existing OS matrix and pinned tool versions for the component that owns them. JNI integration belongs to Java tests because the JNI bridge is packaged and exercised through the Java adapter.

### Pass selected coverage scopes to the gate

The full gate currently requires three-platform Rust reports, Linux FFI consumer data, and all existing adapter reports. Add an explicit set of required scopes rather than infer missing reports from filesystem state. Full mode requires every current scope. Adapter-only modes require only the selected adapter reports and thresholds. C consumer mode requires the existing FFI coverage path. All selected code remains subject to its existing scope threshold and the 95% changed-production-source threshold. Missing expected files fail.

### Validate authorized skips centrally

The Summary already sees all `needs` results and is the final required job. Make its policy plan-driven: actual success is required for selected jobs, and `skipped` is accepted only if the validated plan excludes that job. Invalid or unavailable plans select full CI wherever possible and cannot result in a passing Summary.

## Rejected Alternatives

- **Workflow-level `paths`/`paths-ignore`**: Can omit the entire workflow/check on a SHA and create pending required checks.
- **Trust changed-file extensions only**: Renames, deletions, manifest/build effects, and unrecognized files can escape the intended component; the design uses explicit path maps and full fallback.
- **Reuse earlier green results**: GitHub checks belong to a specific commit SHA; this cannot meet the latest-SHA status requirement.
- **Treat all skipped jobs as success**: Hides a mistakenly skipped selected job; only a central, validated plan may authorize a skip.
- **Download and require every coverage artifact after narrow jobs**: Fails whenever an intentionally omitted producer did not run. Scope-aware aggregation preserves fail-closed behavior for selected scopes.

## Sources

- GitHub, [Troubleshooting required status checks](https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/troubleshooting-required-status-checks).
- GitHub, [Control jobs with conditions](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-jobs-with-conditions).
- Repository current state at base `9e51a083be06ec66230cb5b15c9bb814d84ebefb`: `.github/workflows/ci.yml`, `scripts/ci_summary.py`, `scripts/coverage_gate.py`, `docs/development/testing.md`, and their tests.
