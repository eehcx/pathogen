mod domain;
mod infrastructure;
mod presentation;
mod use_cases;

use presentation::run_tui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_tui()?;
    Ok(())
}
