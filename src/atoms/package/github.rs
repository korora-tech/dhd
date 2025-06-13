use super::{PackageProvider, command_exists};
use std::process::Command;
use std::path::PathBuf;
use std::fs;
use serde_json;

pub struct GitHubProvider;

impl GitHubProvider {
    /// Parse GitHub repository from package spec
    /// Format: owner/repo or owner/repo@version or owner/repo:binary_name or owner/repo:binary_name@version
    fn parse_package_spec(&self, package: &str) -> Result<(String, String, Option<String>, Option<String>), String> {
        let parts: Vec<&str> = package.split('@').collect();
        let repo_and_binary = parts[0];
        let version = parts.get(1).map(|v| v.to_string());
        
        // Check if binary name is specified with :
        let (repo_path, binary_name) = if let Some(colon_pos) = repo_and_binary.find(':') {
            let (repo, binary) = repo_and_binary.split_at(colon_pos);
            (repo, Some(binary[1..].to_string()))
        } else {
            (repo_and_binary, None)
        };
        
        let repo_parts: Vec<&str> = repo_path.split('/').collect();
        if repo_parts.len() != 2 {
            return Err(format!("Invalid GitHub package format. Expected owner/repo[:binary][@version], got: {}", package));
        }
        
        Ok((repo_parts[0].to_string(), repo_parts[1].to_string(), version, binary_name))
    }
    
    /// Get the installation directory for GitHub releases
    fn get_install_dir(&self) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".local").join("bin")
    }
    
    /// Fetch release information from GitHub API
    fn fetch_release_info(&self, owner: &str, repo: &str, version: Option<&str>) -> Result<serde_json::Value, String> {
        let url = match version {
            Some(v) => format!("https://api.github.com/repos/{}/{}/releases/tags/{}", owner, repo, v),
            None => format!("https://api.github.com/repos/{}/{}/releases/latest", owner, repo),
        };
        
        let output = Command::new("curl")
            .args(&["-s", "-H", "Accept: application/vnd.github.v3+json", &url])
            .output()
            .map_err(|e| format!("Failed to fetch release info: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("Failed to fetch release info from GitHub"));
        }
        
        let response = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse GitHub API response: {}", e))
    }
    
    /// Detect the appropriate asset for the current platform
    fn detect_asset(&self, assets: &serde_json::Value) -> Result<(String, String), String> {
        let assets = assets.as_array()
            .ok_or_else(|| "Invalid assets format".to_string())?;
        
        // Get system architecture
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        
        // Common patterns for different platforms
        let patterns = match (os, arch) {
            ("linux", "x86_64") => vec!["linux-amd64", "linux-x86_64", "linux_amd64", "x86_64-unknown-linux"],
            ("linux", "aarch64") => vec!["linux-arm64", "linux-aarch64", "linux_arm64", "aarch64-unknown-linux"],
            ("macos", "x86_64") => vec!["darwin-amd64", "darwin-x86_64", "macos-x86_64", "x86_64-apple-darwin"],
            ("macos", "aarch64") => vec!["darwin-arm64", "darwin-aarch64", "macos-arm64", "aarch64-apple-darwin"],
            _ => vec![],
        };
        
        // Try to find a matching asset
        for asset in assets {
            let name = asset["name"].as_str().unwrap_or("");
            let download_url = asset["browser_download_url"].as_str().unwrap_or("");
            
            // Skip source archives and package formats we don't handle
            if name.ends_with(".sig") || name.ends_with(".asc") ||
               name.ends_with(".deb") || name.ends_with(".rpm") || 
               name.ends_with(".apk") || name.ends_with(".snap") ||
               name == "source.tar.gz" || name == "source.zip" {
                continue;
            }
            
            // Check if the asset name matches any of our patterns
            for pattern in &patterns {
                if name.to_lowercase().contains(pattern) {
                    return Ok((name.to_string(), download_url.to_string()));
                }
            }
        }
        
        // Fallback: look for generic binary names
        for asset in assets {
            let name = asset["name"].as_str().unwrap_or("");
            let download_url = asset["browser_download_url"].as_str().unwrap_or("");
            
            // Check for common binary archive extensions
            if name.ends_with(".tar.gz") || name.ends_with(".zip") || name.ends_with(".tar.xz") {
                return Ok((name.to_string(), download_url.to_string()));
            }
        }
        
        Err(format!("No suitable asset found for {} {}", os, arch))
    }
    
    /// Extract the downloaded archive
    fn extract_archive(&self, archive_path: &str, dest_dir: &str) -> Result<(), String> {
        let output = if archive_path.ends_with(".tar.gz") || archive_path.ends_with(".tgz") {
            Command::new("tar")
                .args(&["-xzf", archive_path, "-C", dest_dir])
                .output()
        } else if archive_path.ends_with(".tar.xz") {
            Command::new("tar")
                .args(&["-xJf", archive_path, "-C", dest_dir])
                .output()
        } else if archive_path.ends_with(".zip") {
            Command::new("unzip")
                .args(&["-q", archive_path, "-d", dest_dir])
                .output()
        } else {
            return Err(format!("Unsupported archive format: {}", archive_path));
        };
        
        match output {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => Err(format!("Failed to extract archive: {}", String::from_utf8_lossy(&output.stderr))),
            Err(e) => Err(format!("Failed to run extraction command: {}", e)),
        }
    }
}

impl PackageProvider for GitHubProvider {
    fn is_available(&self) -> bool {
        // GitHub provider requires curl and either tar or unzip
        command_exists("curl") && (command_exists("tar") || command_exists("unzip"))
    }
    
    fn is_package_installed(&self, package: &str) -> Result<bool, String> {
        let (_, repo, _, binary_name) = self.parse_package_spec(package)?;
        let install_dir = self.get_install_dir();
        
        // Use specified binary name or default to repo name
        let binary_name = binary_name.unwrap_or_else(|| repo.clone());
        let binary_path = install_dir.join(&binary_name);
        
        Ok(binary_path.exists())
    }
    
    fn install_package(&self, package: &str) -> Result<(), String> {
        let (owner, repo, version, binary_name_override) = self.parse_package_spec(package)?;
        
        // Fetch release information
        let release_info = self.fetch_release_info(&owner, &repo, version.as_deref())?;
        
        // Get assets
        let assets = &release_info["assets"];
        let (asset_name, download_url) = self.detect_asset(assets)?;
        
        // Create installation directory
        let install_dir = self.get_install_dir();
        fs::create_dir_all(&install_dir)
            .map_err(|e| format!("Failed to create install directory: {}", e))?;
        
        // Download the asset
        let temp_dir = std::env::temp_dir();
        let download_path = temp_dir.join(&asset_name);
        
        let output = Command::new("curl")
            .args(&["-L", "-o", &download_path.to_string_lossy(), &download_url])
            .output()
            .map_err(|e| format!("Failed to download asset: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("Failed to download asset: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        // Extract if it's an archive
        if asset_name.ends_with(".tar.gz") || asset_name.ends_with(".zip") || asset_name.ends_with(".tar.xz") {
            let extract_dir = temp_dir.join(format!("{}-extract", repo));
            fs::create_dir_all(&extract_dir)
                .map_err(|e| format!("Failed to create extraction directory: {}", e))?;
            
            self.extract_archive(&download_path.to_string_lossy(), &extract_dir.to_string_lossy())?;
            
            // Determine the binary name to install as
            let install_binary_name = binary_name_override.as_ref().unwrap_or(&repo);
            
            // Find the binary in the extracted files
            // Common locations: root, bin/, usr/bin/, or named after the repo
            let possible_binaries = vec![
                extract_dir.join(&repo),
                extract_dir.join("bin").join(&repo),
                extract_dir.join("usr").join("bin").join(&repo),
                extract_dir.join(&repo).with_extension(""),
                // If a binary name override is specified, also look for that
                extract_dir.join(install_binary_name),
                extract_dir.join("bin").join(install_binary_name),
                extract_dir.join("usr").join("bin").join(install_binary_name),
            ];
            
            let mut binary_found = false;
            for possible_binary in possible_binaries {
                if possible_binary.exists() {
                    let dest = install_dir.join(install_binary_name);
                    fs::copy(&possible_binary, &dest)
                        .map_err(|e| format!("Failed to copy binary: {}", e))?;
                    
                    // Make it executable
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let mut perms = fs::metadata(&dest)
                            .map_err(|e| format!("Failed to read permissions: {}", e))?
                            .permissions();
                        perms.set_mode(0o755);
                        fs::set_permissions(&dest, perms)
                            .map_err(|e| format!("Failed to set permissions: {}", e))?;
                    }
                    
                    binary_found = true;
                    break;
                }
            }
            
            // Clean up
            let _ = fs::remove_dir_all(&extract_dir);
            let _ = fs::remove_file(&download_path);
            
            if !binary_found {
                return Err(format!("Could not find binary '{}' in extracted archive", repo));
            }
        } else {
            // Assume it's a binary, copy it (not move, to handle cross-device scenarios)
            let install_binary_name = binary_name_override.as_ref().unwrap_or(&repo);
            let dest = install_dir.join(install_binary_name);
            fs::copy(&download_path, &dest)
                .map_err(|e| format!("Failed to copy binary: {}", e))?;
            
            // Make it executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&dest)
                    .map_err(|e| format!("Failed to read permissions: {}", e))?
                    .permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&dest, perms)
                    .map_err(|e| format!("Failed to set permissions: {}", e))?;
            }
            
            // Clean up the download
            let _ = fs::remove_file(&download_path);
        }
        
        Ok(())
    }
    
    fn uninstall_package(&self, package: &str) -> Result<(), String> {
        let (_, repo, _, binary_name) = self.parse_package_spec(package)?;
        let install_dir = self.get_install_dir();
        
        // Use specified binary name or default to repo name
        let binary_name = binary_name.unwrap_or_else(|| repo.clone());
        let binary_path = install_dir.join(&binary_name);
        
        if binary_path.exists() {
            fs::remove_file(&binary_path)
                .map_err(|e| format!("Failed to remove binary: {}", e))?;
            Ok(())
        } else {
            Err(format!("Package {} is not installed", package))
        }
    }
    
    fn update(&self) -> Result<(), String> {
        // GitHub releases don't have a traditional update mechanism
        Ok(())
    }
    
    fn name(&self) -> &str {
        "github"
    }
    
    fn install_command(&self) -> Vec<String> {
        vec!["github".to_string(), "install".to_string()]
    }
}