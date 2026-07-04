# AM001 Track B Night 2 Build Report

## B0 — Apply Rejection Exit Fix

### Workstream

- Fixed `am apply` normal rejection exits so they no longer bubble an `anyhow::Error` out of `main`.
- Rejection outcomes now print exactly one stderr line and call `std::process::exit(1)`:
  - `EG-1 file rejected before apply`
  - `EG-1 file has structural rejection`
  - `EG-1 apply completed with rejected events`
- Extended `apply_rejection_exit_prints_one_clean_error_line` to assert the same one-line stderr behavior with `RUST_BACKTRACE` unset and with `RUST_BACKTRACE=1`.

### Commands

```text
$ cargo fmt && cargo test --test beval_b02 apply_rejection_exit_prints_one_clean_error_line -- --nocapture
test apply_rejection_exit_prints_one_clean_error_line ... ok
test result: ok. 1 passed; 0 failed
```

```text
$ cargo fmt --check && cargo clippy -- -D warnings && cargo test
69 tests passed; 0 failed
```

### Files Changed

- `src/cli/commands.rs`
- `tests/beval_b02.rs`
- `docs/BUILD_REPORT_NIGHT2.md`

### Deviations

None.

### Known Limitations

None for B0.

### Next

B03 context compiler.
