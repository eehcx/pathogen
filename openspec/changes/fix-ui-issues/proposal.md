# Proposal: Fix UI Issues

## Intent

Fix three bugs in the nftables-tui application that affect UI correctness and code quality:
1. Duplicate rendering of RateLimitList view
2. Rate limit rules incorrectly shown as dropped traffic in the rules list
3. Debug DEV messages left in production code

## Scope

### In Scope
- Remove duplicate `rate_limit_list::render_rate_limit_list` call in `src/presentation/ui.rs`
- Fix `get_all_rules()` in `src/infrastructure/cli_repository.rs` to exclude rate limit rules from the dropped traffic list (or categorize them correctly)
- Remove `println!("[DEV]...")` debug statements from production code

### Out of Scope
- Refactoring the logging infrastructure
- Adding proper logging framework
- Testing changes (manual verification only)

## Approach

1. **Duplicate rendering fix**: Remove line 121 in `src/presentation/ui.rs` - the second call to `rate_limit_list::render_rate_limit_list` is redundant since it's already called on line 119.

2. **Rate limit classification fix**: In `src/infrastructure/cli_repository.rs:80-87`, change the handling of `tui-ratelimit-` comments. Currently they are treated as dropped traffic (`is_drop = true`), but rate limit rules should be categorized separately or not shown in the dropped traffic view. Options:
   - Skip rate limit rules entirely in `get_all_rules()`
   - Add a new `Action::RateLimit` variant
   - Set `is_drop = false` so they appear as accepted traffic

3. **Remove DEV messages**: Delete or conditionally compile the `println!("[DEV]...")` statements at lines 28, 51-52 in `src/infrastructure/cli_repository.rs`. These debug statements should not be in production code.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/presentation/ui.rs:121` | Modified | Remove duplicate render call |
| `src/infrastructure/cli_repository.rs:80-87` | Modified | Fix rate limit rule categorization |
| `src/infrastructure/cli_repository.rs:28,51-52` | Modified | Remove debug println statements |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Accidental removal of needed code | Low | Review each change carefully |
| Rate limit rules no longer visible | Low | Verify they appear in correct view after fix |

## Rollback Plan

Revert changes using git:
```bash
git checkout HEAD -- src/presentation/ui.rs src/infrastructure/cli_repository.rs
```

## Dependencies

- None - all changes are local to the codebase

## Success Criteria

- [ ] RateLimitList renders only once in the UI
- [ ] Rate limit rules are no longer incorrectly shown as dropped traffic
- [ ] No `[DEV]` messages appear in production output
- [ ] Application compiles without errors
