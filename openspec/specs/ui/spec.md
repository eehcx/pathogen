# UI Specification - RateLimitList Rendering

## Requirements

### Requirement: RateLimitList View MUST Be Rendered

When the application enters `AppMode::RateLimitList`, the system SHALL render the rate limit rules list into the main content area between the header and footer.

#### Scenario: RateLimitList mode displays list

- GIVEN the application is running and the user navigates to the rate limit rules list
- WHEN the application mode is set to `AppMode::RateLimitList`
- THEN the rate limit rules list SHALL be rendered in the main content area
- AND the header SHALL display "Traffic Control" with subtitle "Manage Rate Limit Rules"
- AND the footer SHALL display navigation help "[↑↓] Navigate  [d] Delete Rule  [m] Menu"

## Requirements

### Requirement: Rate Limit List Selection State MUST Persist

The rate limit list widget state SHALL use a mutable reference to the application's state, not a clone, so that user selection changes persist after each render cycle.

(Previously: The widget received a cloned state, causing selections to be lost after each render)

#### Scenario: User navigates to different rate limit rule

- GIVEN the rate limit rules list is displayed with multiple rules
- WHEN the user presses the Down arrow key to select a different rule
- THEN the selected item SHALL change to the new rule
- AND the selection SHALL persist after the next render cycle

#### Scenario: Selection persists across menu navigation

- GIVEN a rate limit rule is currently selected
- WHEN the user presses 'm' to return to the menu and then navigates back to RateLimitList
- THEN the previously selected rule SHALL remain selected
