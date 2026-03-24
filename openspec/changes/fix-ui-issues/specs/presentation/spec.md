# Presentation Specification

## Purpose

This specification covers the presentation layer of the nftables-tui application, specifically the UI rendering logic that displays firewall rules and rate limits to users.

## Requirements

### Requirement: Rate Limit List Must Not Render Duplicate Views

The system SHALL NOT render the rate limit list component more than once in a single view render cycle.

#### Scenario: Rate limit list renders exactly once

- GIVEN the user navigates to the Traffic Control view
- WHEN the UI renders the rate limit list component
- THEN the rate_limit_list::render_rate_limit_list function MUST be called exactly once
- AND the rate limit list MUST appear only in its designated area

#### Scenario: Duplicate render does not occur

- GIVEN the application is running in any state
- WHEN the UI render function executes for the Traffic Control view
- THEN the render function MUST NOT call rate_limit_list::render_rate_limit_list multiple times for the same area

### Requirement: View Components Must Render in Correct Order

The system MUST render view components in the order defined by the layout constraints.

#### Scenario: Header renders before content

- GIVEN the user is on the Traffic Control view
- WHEN the frame is rendered
- THEN the ASCII header MUST render first in the designated header area
- AND the rate limit list MUST render second in the content area
- AND the footer MUST render last in the footer area