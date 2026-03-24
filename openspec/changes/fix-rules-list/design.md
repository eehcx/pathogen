# Design: Fix Empty Rules List

## Technical Approach

The fix modifies `cli_repository.rs:get_all_rules()` to:
1. Remove the filter that only returns rules with "tui-blocked-" comments
2. Parse the `expr` field in `NftRule` to determine the actual action (accept/drop)
3. Skip rules with "tui-ratelimit-" comments (managed separately)

## Architecture Decisions

### Decision: Parse expression for action instead of relying on comment prefix

**Choice**: Parse the `expr` array in nftables JSON output to find the action
**Alternatives considered**: Keep using comment prefix only
**Rationale**: Allows displaying ALL rules, not just TUI-created ones

### Decision: Default to Accept for rules without clear action

**Choice**: When expression doesn't contain explicit accept/drop, default to Action::Accept
**Alternatives considered**: Return error, skip rule
**Rationale**: Most firewall rules are accept rules; prevents empty list

## Data Flow

```
nft_list_rules.sh
       │
       ▼
   JSON Output
       │
       ▼
NftablesOutput (serde_json)
       │
       ▼
  Filter: skip "tui-ratelimit-"
       │
       ▼
Parse expr for action (accept/drop)
       │
       ▼
   Vec<Rule>
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src/infrastructure/cli_repository.rs` | Modify | Update `get_all_rules()` to return all rules and parse expression for action |

## Interfaces / Contracts

The `Rule` struct in `src/domain/entities/rule.rs` already has an `action: Action` field. The change only affects how this field is populated.

```rust
// Action enum already exists
pub enum Action {
    Accept,
    Drop,
}
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | get_all_rules() filtering logic | Test with mocked nftables JSON output |
| Integration | Full rule display flow | Run app and verify rules appear |

## Migration / Rollout

No migration required. This is a purely additive change that shows more rules.

## Open Questions

- [ ] None - approach is straightforward
