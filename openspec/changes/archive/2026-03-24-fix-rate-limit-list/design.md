# Design: Fix RateLimitList Rendering and State Bug

## Technical Approach

Fix two bugs in the RateLimitList feature: (1) add missing render call in ui.rs to display the list, and (2) fix state clone bug to persist selections. This follows the existing pattern used by RulesList.

## Architecture Decisions

### Decision: Use existing view rendering pattern

**Choice**: Add `rate_limit_list::render_rate_limit_list(frame, app, chunks[1])` call in ui.rs, following the exact pattern from RulesList.

**Alternatives considered**: Creating a new render function wrapper or using a different component approach.

**Rationale**: RulesList already implements this pattern correctly at line 40 of ui.rs. Following the existing pattern ensures consistency and reduces risk.

### Decision: Use mutable reference instead of clone for state

**Choice**: Change `&mut app.rate_limit_list.state.clone()` to `&mut app.rate_limit_list.state` in rate_limit_list.rs.

**Alternatives considered**: Wrapping state in Rc<RefCell<>> or implementing a different state management approach.

**Rationale**: The clone creates a copy that is dropped after render, losing all selection changes. Using a mutable reference follows the same pattern used by RulesList and persists state changes.

## Data Flow

```
AppState
   │
   ├── mode = AppMode::RateLimitList
   │
   └── rate_limit_list: RateLimitListState
         ├── items: Vec<Rule>
         └── state: ListState ──→ render_stateful_widget (mutable reference)
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src/presentation/ui.rs` | Modify | Add render call at line ~159, between header and footer |
| `src/presentation/views/rate_limit_list.rs` | Modify | Change line 89 from `.clone()` to direct mutable reference |

## Interfaces / Contracts

```rust
// ui.rs - Add this line in AppMode::RateLimitList case (after header, before footer):
rate_limit_list::render_rate_limit_list(frame, app, chunks[1]);

// rate_limit_list.rs - Line 89 change:
frame.render_stateful_widget(list, area, &mut app.rate_limit_list.state.clone());
// BECOMES:
frame.render_stateful_widget(list, area, &mut app.rate_limit_list.state);
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Manual | RateLimitList renders | Navigate to RateLimitList and verify list appears |
| Manual | Selection persists | Select different rule, navigate away, return, verify selection |

## Migration / Rollback

No migration required. This is a bug fix with simple, reversible changes.

Rollback:
```bash
git checkout -- src/presentation/ui.rs src/presentation/views/rate_limit_list.rs
```

## Open Questions

None - the implementation is straightforward and follows existing patterns.
