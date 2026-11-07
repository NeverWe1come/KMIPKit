# KMIPKIT-0051 fixture reconciliation evidence

## Red

After merging the final 0050 predecessor (`09244e62860ae581484e55514b8aa245c6429d15`),
`python -B -m unittest discover -s tools/api_manifest/tests -p 'test_t102_*.py' -q`
ran 399 tests in 256.760 seconds and reported 107 failures.

The current merged manifest preserves all predecessor declarations without
changes and adds precisely the 930 already blocked 0051 validation rows.
No other manifest section differs from the final predecessor. Of these rows,
902 retain candidate owner `KMIPKIT-0043`, 24 retain `KMIPKIT-0009`, and four
retain `KMIPKIT-0044` as assigned by the official generic reconciler. Their
four-language capability remains `public_binding_surface`, with no parity
mapping or readiness promotion.

Failures expose three stale fixture assumptions: the shared 0043 candidate
count is zero; standard-trait exclusions collect newly added project-specific
`ValidateKmipRequest::{SCHEMA,validate}` members; and fixed non-target
declaration hashes predate the 930 additions. The exact Rust and binding
audits already pass against the merged manifest. Fixture corrections must
preserve the manifest and its ownership, retain all new rows in non-target
fingerprints, and separately pin the new validation members as blocked.

The subsequent cross-language validation reconciliation remains assigned to
KMIPKIT-0053. These Rust validation rows do not claim C, Java, or Python
validation parity.

The additional focused regression module
`tools.api_manifest.tests.test_t102_validation_route_inventory` ran three
tests and failed exactly once: its explicit 902-candidate assertion observed
the stale zero constant. The exact 252 validation trait routes, existing
owners, all 930 blocked additions, absence of parity promotions, and negative
selector cases passed as characterization of the existing manifest.

## Green

The fixtures now keep the 902 current 0043 candidates distinct from the
historical macro-lot IDs. Standard-trait assertions omit only the exact 252
existing validation member routes recognized by the shared test selector.
Separate tests pin every member, its existing owner, and its blocked state;
future members, types, and packages are not recognized by that selector.
The three non-target declaration fingerprints retain all 930 additions.
The manifest and product source remain unchanged by these corrections.

- The focused validation/Export/Get/Revoke modules passed nine tests.
- The final four residual fixture cases and new inventory module passed all
  17 tests in their five focused modules.
- Full T102 discovery passed all 402 tests (the original 399 plus three new
  regressions) in 111.376 seconds.
- The current shared inventory fixture first failed with the expected
  `23585 != 22655` stale predecessor count. After refreshing its observed and
  matched counts, the focused generic-reconciler, shared-inventory, and exact
  manifest-inventory checks passed all 11 tests.
- The affected compiled C ABI coverage suite passed all 50 tests.

## Refactor

Predicate formatting in 18 historical fixture modules was clarified after
the Green commit. Comparing parsed Python ASTs against `d5b78cc28` confirmed
that every module has identical executable structure. The five focused
modules passed all 17 tests again, and `git diff --check` passed.

The final API generator check verified all 12 outputs, the feature-specific
shared inventory check passed, and `cargo fmt --all --check` passed. Runtime
source and manifest inputs were unchanged after the passing Rust and binding
audits, so those snapshots remain applicable.

Development evidence is separated into the merge/Red commit `8079b65ed`, the
focused regression Red commit `788c57004`, the Green commit `d5b78cc28`, and
the subsequent fixture-format Refactor commit.
