#[cfg(test)]
mod tests {
    use dhd::atoms::package::PackageManager;

    #[test]
    fn test_github_provider_creation() {
        let provider = PackageManager::GitHub.get_provider();
        assert_eq!(provider.name(), "github");
    }

    #[test]
    fn test_github_package_spec_parsing() {
        // This test validates the internal parsing logic
        // We can't directly test the private method, but we can test through the public API
        let provider = PackageManager::GitHub.get_provider();
        
        // Valid formats should not error
        assert!(provider.is_package_installed("owner/repo").is_ok());
        assert!(provider.is_package_installed("owner/repo@v1.0.0").is_ok());
        assert!(provider.is_package_installed("owner/repo:binary").is_ok());
        assert!(provider.is_package_installed("owner/repo:binary@v1.0.0").is_ok());
    }

    #[test]
    fn test_github_provider_is_available() {
        let provider = PackageManager::GitHub.get_provider();
        // The provider should be available if curl and either tar or unzip are present
        // This test will pass/fail based on the test environment
        let is_available = provider.is_available();
        
        // Check if the required tools exist
        let has_curl = std::process::Command::new("which")
            .arg("curl")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
            
        let has_tar = std::process::Command::new("which")
            .arg("tar")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
            
        let has_unzip = std::process::Command::new("which")
            .arg("unzip")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
            
        assert_eq!(is_available, has_curl && (has_tar || has_unzip));
    }

    #[test]
    fn test_github_install_command() {
        let provider = PackageManager::GitHub.get_provider();
        let cmd = provider.install_command();
        assert_eq!(cmd, vec!["github", "install"]);
    }

    #[test]
    fn test_package_manager_from_str() {
        use std::str::FromStr;
        
        let manager = PackageManager::from_str("github").unwrap();
        assert!(matches!(manager, PackageManager::GitHub));
        
        let manager_lower = PackageManager::from_str("GitHub").unwrap();
        assert!(matches!(manager_lower, PackageManager::GitHub));
    }

    #[test]
    fn test_invalid_package_format() {
        let provider = PackageManager::GitHub.get_provider();
        
        // Invalid formats should error
        assert!(provider.is_package_installed("invalid-format").is_err());
        assert!(provider.is_package_installed("too/many/slashes").is_err());
        assert!(provider.is_package_installed("").is_err());
    }

    #[test]
    fn test_package_not_installed() {
        let provider = PackageManager::GitHub.get_provider();
        
        // Check a package that's unlikely to be installed
        match provider.is_package_installed("nonexistent/repo") {
            Ok(installed) => assert!(!installed),
            Err(_) => panic!("Should not error on checking non-existent package"),
        }
    }
}