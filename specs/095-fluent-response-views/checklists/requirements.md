# KMIPKIT-0095 Requirements Checklist

- [x] The response renderer includes the common response header and ordered
  batch under the KMIP ResponseMessage root.
- [x] TTLV, JSON and XML paths use existing codec validation and caller limits.
- [x] Rendered data leaves only in a zeroizing return owner.
- [x] Encoding errors contain no body or vendor payload.
- [x] The renderer has no transport handle or request execution permit.
- [x] Every new response-view Rust export is declared or has an explicit
  language-level exclusion; generated manifest check passes.
- [ ] C, Java and Python wrappers preserve their native ownership rules.
- [ ] Required pull request CI is green.
