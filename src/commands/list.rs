use crate::{Result, modules::loader::load_modules_from_directory};
use std::path::PathBuf;

pub fn execute(modules_path: Option<PathBuf>) -> Result<()> {
    let search_path = modules_path
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    println!("Scanning for modules in: {}", search_path.display());
    println!();

    let modules = load_modules_from_directory(&search_path)?;

    if modules.is_empty() {
        println!("No modules found.");
        return Ok(());
    }

    println!("Available modules:");
    println!();

    for module in &modules {
        println!(
            "  {} - {}",
            module.name,
            module.description.as_deref().unwrap_or("No description")
        );
    }

    println!();
    println!("Total: {} modules", modules.len());

    Ok(())
}
