# Infrastructure Specification

## Purpose

This specification covers the infrastructure layer of the nftables-tui application, specifically the CLI repository that interfaces with the nftables system and the rule parsing logic.

## Requirements

### Requirement: Rate Limit Rules Must Not Be Categorized as Dropped Traffic

The system MUST NOT classify rate limit rules (identified by comments starting with "tui-ratelimit-") as dropped traffic in the firewall rules list.

#### Scenario: Rate limit rules excluded from dropped traffic

- GIVEN nftables contains rules with comments starting with "tui-ratelimit-"
- WHEN get_all_rules() is called
- THEN the returned rules with "tui-ratelimit-" comments MUST NOT have is_drop set to true
- AND rate limit rules MUST either be excluded from the results or categorized differently

#### Scenario: Standard blocked rules still categorized as dropped

- GIVEN nftables contains rules with comments starting with "tui-blocked-"
- WHEN get_all_rules() is called
- THEN the returned rules with "tui-blocked-" comments MUST have is_drop set to true

### Requirement: No Debug Output in Production Code

The system MUST NOT produce debug output via println! statements during normal operation in production.

#### Scenario: Production mode has no DEV messages

- GIVEN the application is running without development mode flags
- WHEN any repository method is executed
- THEN no "[DEV]" prefixed messages MUST be printed to stdout

#### Scenario: Development mode may emit debug output

- GIVEN the application is running with development mode enabled (PATHOGEN_DEV_MODE environment variable)
- WHEN any repository method is executed
- THEN debug output MAY be printed for development purposes

### Requirement: Firewall Rules List Must Only Contain Firewall Rules

The system MUST only return firewall rules in the get_all_rules() method, excluding rate limiting rules from the main rules list.

#### Scenario: Only firewall rules returned

- GIVEN nftables contains both firewall rules (tui-blocked-) and rate limit rules (tui-ratelimit-)
- WHEN get_all_rules() is called
- THEN the returned list MUST contain only rules with "tui-blocked-" or no tui- prefix comments
- AND rules with "tui-ratelimit-" comments MUST NOT appear in the returned list