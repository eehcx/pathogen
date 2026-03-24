# Delta for Infrastructure

## MODIFIED Requirements

### Requirement: Firewall Rules List Must Only Contain Firewall Rules

The system MUST return ALL firewall rules from nftables in the get_all_rules() method, excluding only rate limiting rules.

(Previously: The system MUST only return firewall rules in the get_all_rules() method, excluding rate limiting rules from the main rules list. Only rules with "tui-blocked-" or no tui- prefix comments were returned.)

#### Scenario: All firewall rules returned including system rules

- GIVEN nftables contains both TUI-created rules (with "tui-blocked-" comments) and system rules (without "tui-" comments)
- WHEN get_all_rules() is called
- THEN the returned list MUST contain all rules regardless of comment content
- AND rules with "tui-ratelimit-" comments MUST NOT appear in the returned list

#### Scenario: Rate limit rules excluded from main list

- GIVEN nftables contains rules with "tui-ratelimit-" comments
- WHEN get_all_rules() is called
- THEN rules with "tui-ratelimit-" comments MUST NOT appear in the returned list

### Requirement: Rule Action Determined by Expression

The system MUST determine the rule action (Accept/Drop) based on the actual nftables expression/action in the rule, not solely from the comment prefix.

#### Scenario: Accept action from expression

- GIVEN nftables contains a rule with an "accept" action in its expression
- WHEN get_all_rules() is called
- THEN the returned rule MUST have action set to Action::Accept

#### Scenario: Drop action from expression

- GIVEN nftables contains a rule with a "drop" or "reject" action in its expression
- WHEN get_all_rules() is called
- THEN the returned rule MUST have action set to Action::Drop

#### Scenario: Default accept for rules without clear action

- GIVEN nftables contains a rule where the expression does not contain a clear accept or drop action
- WHEN get_all_rules() is called
- THEN the returned rule MUST have action set to Action::Accept

#### Scenario: TUI-blocked rules maintain drop action

- GIVEN nftables contains a rule with comment "tui-blocked-tcp-8080"
- WHEN get_all_rules() is called
- THEN the returned rule MUST have action set to Action::Drop (for backward compatibility with existing blocked rules)
