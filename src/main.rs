mod domain;
mod infrastructure;
mod presentation;
mod use_cases;

use presentation::run_tui;
use std::process::Command;

fn is_root() -> bool {
    let output = Command::new("id").arg("-u").output();
    if let Ok(out) = output {
        let uid = String::from_utf8_lossy(&out.stdout).trim().to_string();
        uid == "0"
    } else {
        false
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !is_root() {
        eprintln!("Error: Pathogen requiere privilegios de superusuario (root) para interactuar con nftables.");
        eprintln!("Por favor, ejecuta el programa usando: sudo {}", std::env::args().next().unwrap_or_else(|| "pathogen".to_string()));
        std::process::exit(1);
    }

    run_tui()?;
    Ok(())
}
