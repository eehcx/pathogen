# Design: Fix UI Issues

## Technical Approach

Fix three isolated bugs in the nftables-tui application:
1. Remove duplicate render call in the RateLimitList view
2. Filter out rate limit rules from the firewall rules list (they are internal tc rules, not dropped traffic)
3. Remove debug DEV messages from production code

## Architecture Decisions

### Decision: Rate Limit Rule Handling

**Choice**: Filter out rate limit rules entirely from `get_all_rules()` by skipping them in the loop
**Alternatives considered**: 
- Adding `Action::RateLimit` variant (requires domain changes, more invasive)
- Setting `is_drop = false` (misleading - they're not "accepted" traffic, they're rate-limited)
- Creating a separate method (adds unnecessary complexity)
**Rationale**: Rate limit rules are internal implementation details managed by `tc` (traffic control), not actual nftables firewall rules. They should not appear in the firewall rules view at all. Simple filtering is the least invasive fix.

### Decision: DEV Message Removal

**Choice**: Remove all `println!("[DEV]...")` statements entirely
**Alternatives considered**: 
- Conditional compilation with `#[cfg(feature = "debug")]` (adds complexity for debug-only code)
- Using a proper logging framework (out of scope per proposal)
**Rationale**: These are clearly debug statements that should never have been in production code. The simplest fix is removal.

## Data Flow

The changes are localized to two files with no impact on data flow:

```
src/presentation/ui.rs          src/infrastructure/cli_repository.rs
┌─────────────────────────────┐   ┌─────────────────────────────────────┐
│ render_rate_limit_list()    │   │ get_all_rules()                     │
│   called on line 119 ✓       │   │   parses nft JSON output             │
│   called on line 121 ✗       │   │   filters: tui-blocked-* → Drop      │
│   └─ Remove line 121         │   │   filters: tui-ratelimit-* → SKIP   │  ← Change
└─────────────────────────────┘   │   filters: println("[DEV]...") → RM  │  ← Change
                                  └─────────────────────────────────────┘
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src/presentation/ui.rs` | Modify | Remove line 121 (duplicate render call) |
| `src/infrastructure/cli_repository.rs` | Modify | Lines 80-87: change `is_drop = true` to skip rate limit rules entirely |
| `src/infrastructure/cli_repository.rs` | Modify | Remove lines 28, 51-52 (DEV println statements) |

## Interfaces / Contracts

No new interfaces required. The `Rule` struct remains unchanged.

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Manual | All three fixes | Build and run the application |

## Migration / Rollback Plan

No migration required. All changes are local to the affected files.

Rollback:
```bash
git checkout HEAD -- src/presentation/ui.rs src/infrastructure/cli_repository.rs
```

## Open Questions

- [x] None - the approach for each fix is straightforward
