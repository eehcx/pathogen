/// Action represents the action to take when a rule matches
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Accept,
    Drop,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Accept => write!(f, "accept"),
            Action::Drop => write!(f, "drop"),
        }
    }
}

impl std::str::FromStr for Action {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "accept" => Ok(Action::Accept),
            "drop" => Ok(Action::Drop),
            _ => Err(format!("Unknown action: {}", s)),
        }
    }
}
