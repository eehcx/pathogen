# Proposal: Fix Empty Rules List

## Intent

The RulesList view in `src/presentation/views/rules.rs` displays an empty list instead of firewall rules. Users cannot see or manage their firewall rules through the TUI.

## Scope

### In Scope
- Fix the filtering logic in `cli_repository.rs:get_all_rules()` to show all rules
- Display rules without TUI-specific comments (regular firewall rules)
- Preserve the existing rate limit rule filtering behavior (show blocked rules, handle rate limits separately)

### Out of Scope
- Modifying the rule creation/blocking workflow
- Changes to shell scripts that interact with nftables

## Approach

The root cause is in `src/infrastructure/cli_repository.rs` lines 66-76 where the parsing logic filters out rules that don't have comments starting with "tui-blocked-". The logic should be modified to:

1. Include ALL rules from the nftables output regardless of comment
2. Determine action (Accept/Drop) based on the actual nftables action in the expression, not just the comment prefix
3. Preserve the rate limit filtering (skip "tui-ratelimit-" comments as they are managed separately)

This requires parsing the `expr` field in `NftRule` to determine the actual action (accept vs drop).

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/infrastructure/cli_repository.rs` | Modified | Fix `get_all_rules()` to show all rules |
| `src/infrastructure/nftables_json.rs` | Modified | May need to extend `NftRule` to parse expressions |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Parsing errors if nftables output format changes | Medium | Add logging for parse failures |
| Displaying rules without actions gracefully | Low | Default to "Accept" for rules without clear action |

## Rollback Plan

1. Revert changes to `cli_repository.rs`
2. The change is purely additive - rolls back to showing only "tui-blocked-" rules

## Dependencies

- None - can be implemented directly

## Success Criteria

- [ ] RulesList view displays all firewall rules (both TUI-created and system rules)
- [ ] Regular accept rules show with "ACCEPT" action
- [ ] Blocked ports show with "DROP" action
- [ ] No rules are silently dropped due to filtering