# Tasks: Fix UI Issues

## Phase 1: Remove Duplicate Render Call

- [x] 1.1 Remove duplicate `rate_limit_list::render_rate_limit_list` call in `src/presentation/ui.rs` line 121
  - Verify that only one call to `render_rate_limit_list` remains (line 119)

## Phase 2: Fix Rate Limit Rule Categorization

- [x] 2.1 Modify `get_all_rules()` in `src/infrastructure/cli_repository.rs` to skip rules with "tui-ratelimit-" comments
  - Find the loop that processes rules (around lines 80-87)
  - Add early continue/skip for rules where comment starts with "tui-ratelimit-"
  - Verify that rate limit rules are not included in the returned list

## Phase 3: Remove DEV Debug Statements

- [x] 3.1 Remove `println!("[DEV]...")` statement at `src/infrastructure/cli_repository.rs` line 28
- [x] 3.2 Remove `println!("[DEV]...")` statements at `src/infrastructure/cli_repository.rs` lines 51-52

## Phase 4: Verification

- [x] 4.1 Build the application: `cargo build`
  - Verify compilation succeeds without errors
- [x] 4.2 Verify all three fixes are applied correctly
  - Check ui.rs: only one render call exists
  - Check cli_repository.rs: no "tui-ratelimit-" rules in get_all_rules output
  - Check cli_repository.rs: no println!("[DEV]"...) statements remain