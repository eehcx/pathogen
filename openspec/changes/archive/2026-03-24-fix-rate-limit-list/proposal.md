# Proposal: Fix RateLimitList Rendering and State Bug

## Intent

Fix two bugs causing the RateLimitList feature to be non-functional: the view is not rendered due to a missing render call, and user selections don't persist due to improper state handling.

## Scope

### In Scope
- Add missing `rate_limit_list::render_rate_limit_list()` call in ui.rs
- Fix state clone bug in rate_limit_list.rs line 89 by using `&mut app.rate_limit_list.state` instead of `.clone()`

### Out of Scope
- Any other UI or state handling issues
- New features for rate limiting

## Approach

1. **Fix render call**: In `src/ui.rs`, add the missing call to `rate_limit_list::render_rate_limit_list()` within the `AppMode::RateLimitList` case block (around line 165), following the pattern used by `RulesList`.

2. **Fix state reference**: In `src/rate_limit_list.rs` line 89, remove the `.clone()` call on state and instead use a mutable reference `&mut app.rate_limit_list.state` to ensure changes persist.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/ui.rs:142-165` | Modified | Add render call in RateLimitList case |
| `src/rate_limit_list.rs:89` | Modified | Change state clone to mutable reference |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Breaking other views | Low | Small, targeted changes following existing patterns |
| Compile errors | Low | Verify against similar patterns in RulesList |

## Rollback Plan

Revert the two file changes using git:
```bash
git checkout -- src/ui.rs src/rate_limit_list.rs
```

## Dependencies

- None - these are local fixes with no external dependencies

## Success Criteria

- [ ] RateLimitList view renders when AppMode::RateLimitList is active
- [ ] Selection changes in rate limit list persist after interaction
- [ ] Code compiles without errors
