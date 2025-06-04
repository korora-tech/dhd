pub mod apt;
pub mod arch;
pub mod brew;
pub mod bun;
pub mod cargo;
pub mod deno;
pub mod flatpak;
pub mod npm;
pub mod paru;
pub mod pkg;
pub mod ports;
pub mod winget;

use super::PackageManager;

/// Get all available system package managers
pub fn all_system_managers() -> Vec<Box<dyn PackageManager>> {
    vec![
        Box::new(apt::Apt),
        Box::new(arch::Pacman),
        Box::new(paru::Paru),
        Box::new(brew::Brew),
        Box::new(flatpak::Flatpak),
        Box::new(pkg::Pkg),
        Box::new(ports::Ports),
        Box::new(winget::Winget),
    ]
}

/// Get a package manager by name
pub fn get_manager_by_name(name: &str) -> Option<Box<dyn PackageManager>> {
    match name.to_lowercase().as_str() {
        "apt" => Some(Box::new(apt::Apt)),
        "pacman" => Some(Box::new(arch::Pacman)),
        "paru" => Some(Box::new(paru::Paru)),
        "aur" => Some(Box::new(paru::Paru)), // alias
        "brew" | "homebrew" => Some(Box::new(brew::Brew)),
        "flatpak" => Some(Box::new(flatpak::Flatpak)),
        "pkg" => Some(Box::new(pkg::Pkg)),
        "ports" => Some(Box::new(ports::Ports)),
        "winget" => Some(Box::new(winget::Winget)),
        "npm" => Some(Box::new(npm::Npm)),
        "bun" => Some(Box::new(bun::Bun)),
        "cargo" => Some(Box::new(cargo::Cargo)),
        "deno" => Some(Box::new(deno::Deno)),
        _ => None,
    }
}
