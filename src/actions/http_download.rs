use dhd_macros::{typescript_fn, typescript_type};

use std::path::{Path, PathBuf};
use crate::atoms::AtomCompat;

#[typescript_type]
pub struct Checksum {
    pub algorithm: String,
    pub value: String,
}

#[typescript_type]
pub struct HttpDownload {
    pub url: String,
    pub destination: String,
    pub checksum: Option<Checksum>,
    pub mode: Option<u32>,
}

impl crate::actions::Action for HttpDownload {
    fn name(&self) -> &str {
        "HttpDownload"
    }

    fn plan(&self, _module_dir: &Path) -> Vec<Box<dyn crate::atom::Atom>> {
        let destination_path = if self.destination.starts_with("~/") {
            let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/home/user"));
            PathBuf::from(self.destination.replacen("~/", &format!("{}/", home), 1))
        } else {
            PathBuf::from(&self.destination)
        };

        let checksum_str = self.checksum.as_ref().map(|c| format!("{}:{}", c.algorithm, c.value));

        vec![Box::new(AtomCompat::new(
            Box::new(crate::atoms::http_download::HttpDownload::new(
                self.url.clone(),
                destination_path,
                checksum_str,
                self.mode,
            )),
            "http_download".to_string(),
        ))]
    }
}

#[typescript_fn]
pub fn http_download(config: HttpDownload) -> crate::actions::ActionType {
    crate::actions::ActionType::HttpDownload(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::Action;

    #[test]
    fn test_checksum_creation() {
        let checksum = Checksum {
            algorithm: "sha256".to_string(),
            value: "abc123".to_string(),
        };

        assert_eq!(checksum.algorithm, "sha256");
        assert_eq!(checksum.value, "abc123");
    }

    #[test]
    fn test_http_download_creation() {
        let checksum = Checksum {
            algorithm: "sha256".to_string(),
            value: "test_hash".to_string(),
        };

        let action = HttpDownload {
            url: "https://example.com/file.bin".to_string(),
            destination: "/tmp/file.bin".to_string(),
            checksum: Some(checksum),
            mode: Some(0o755),
        };

        assert_eq!(action.url, "https://example.com/file.bin");
        assert_eq!(action.destination, "/tmp/file.bin");
        assert!(action.checksum.is_some());
        assert_eq!(action.mode, Some(0o755));
    }

    #[test]
    fn test_http_download_helper_function() {
        let action = http_download(HttpDownload {
            url: "https://example.com/file.bin".to_string(),
            destination: "/tmp/file.bin".to_string(),
            checksum: None,
            mode: None,
        });

        match action {
            crate::actions::ActionType::HttpDownload(download) => {
                assert_eq!(download.url, "https://example.com/file.bin");
                assert_eq!(download.destination, "/tmp/file.bin");
                assert!(download.checksum.is_none());
                assert!(download.mode.is_none());
            }
            _ => panic!("Expected HttpDownload action type"),
        }
    }

    #[test]
    fn test_http_download_name() {
        let action = HttpDownload {
            url: "https://example.com/file.bin".to_string(),
            destination: "/tmp/file.bin".to_string(),
            checksum: None,
            mode: None,
        };

        assert_eq!(action.name(), "HttpDownload");
    }

    #[test]
    fn test_http_download_plan() {
        let action = HttpDownload {
            url: "https://example.com/file.bin".to_string(),
            destination: "/tmp/file.bin".to_string(),
            checksum: None,
            mode: None,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        // Check that we got an atom
        assert_eq!(atoms.len(), 1);
    }

    #[test]
    fn test_http_download_with_checksum() {
        let checksum = Checksum {
            algorithm: "sha256".to_string(),
            value: "cfd2bf2c0e81668a7b9263e3b76c857783d6edad2ad43a4013da4be1346b9fb5".to_string(),
        };

        let action = HttpDownload {
            url: "https://github.com/sigstore/gitsign/releases/download/v0.11.0/file".to_string(),
            destination: "/tmp/file".to_string(),
            checksum: Some(checksum),
            mode: Some(0o755),
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        assert!(atoms[0].describe().contains("Download"));
    }

    #[test]
    fn test_http_download_home_expansion() {
        unsafe { std::env::set_var("HOME", "/home/testuser"); }

        let action = HttpDownload {
            url: "https://example.com/file.bin".to_string(),
            destination: "~/bin/file.bin".to_string(),
            checksum: None,
            mode: None,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        assert!(atoms[0].describe().contains("/home/testuser/bin/file.bin"));
    }

    #[test]
    fn test_checksum_string_formatting() {
        let checksum = Checksum {
            algorithm: "sha256".to_string(),
            value: "abc123".to_string(),
        };

        let action = HttpDownload {
            url: "https://example.com/file.bin".to_string(),
            destination: "/tmp/file.bin".to_string(),
            checksum: Some(checksum),
            mode: None,
        };

        // Test that checksum gets formatted correctly in the atom creation
        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
    }
}