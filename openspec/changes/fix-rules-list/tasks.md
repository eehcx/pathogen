# Tasks: Fix Empty Rules List

## Phase 1: Core Implementation

- [x] 1.1 Modify `src/infrastructure/cli_repository.rs:get_all_rules()` to remove the filter that only keeps "tui-blocked-" rules — include all rules except "tui-ratelimit-"
- [x] 1.2 Add logic to parse the `expr` field in `NftRule` to determine action (accept/drop) from nftables expression
- [x] 1.3 Default to `Action::Accept` for rules without a clear drop action in expression (preserve existing behavior for tui-blocked- rules to show as Drop)

## Phase 2: Build and Verify

- [x] 2.1 Run `cargo build` to verify the code compiles
- [x] 2.2 Run `cargo check` for any warnings
- [ ] 2.3 Verify the rules list populates with actual firewall rules

## Phase 3: Testing

- [ ] 3.1 Test with running application to confirm rules display in RulesList view
- [ ] 3.2 Verify both TUI-created rules and system rules appear