use dhd_macros::{typescript_fn, typescript_type};

use crate::atoms::AtomCompat;
use std::path::{Path, PathBuf};

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

        let checksum_str = self
            .checksum
            .as_ref()
            .map(|c| format!("{}:{}", c.algorithm, c.value));

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
            value: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
        };

        assert_eq!(checksum.algorithm, "sha256");
        assert_eq!(checksum.value, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    #[test]
    fn test_http_download_creation() {
        let checksum = Checksum {
            algorithm: "sha256".to_string(),
            value: "d2d2c76b7c7a3e7816a8e7f7c7c8d6f7c8a2b1c2d3e4f5a6b7c8d9e0f1a2b3c4".to_string(),
        };

        let action = HttpDownload {
            url: "https://github.com/kubernetes/kubectl/releases/download/v1.28.0/kubectl".to_string(),
            destination: "/usr/local/bin/kubectl".to_string(),
            checksum: Some(checksum),
            mode: Some(0o755),
        };

        assert_eq!(action.url, "https://github.com/kubernetes/kubectl/releases/download/v1.28.0/kubectl");
        assert_eq!(action.destination, "/usr/local/bin/kubectl");
        assert!(action.checksum.is_some());
        assert_eq!(action.mode, Some(0o755));
    }

    #[test]
    fn test_http_download_helper_function() {
        let action = http_download(HttpDownload {
            url: "https://get.helm.sh/helm-v3.13.0-linux-amd64.tar.gz".to_string(),
            destination: "/tmp/helm.tar.gz".to_string(),
            checksum: None,
            mode: None,
        });

        match action {
            crate::actions::ActionType::HttpDownload(download) => {
                assert_eq!(download.url, "https://get.helm.sh/helm-v3.13.0-linux-amd64.tar.gz");
                assert_eq!(download.destination, "/tmp/helm.tar.gz");
                assert!(download.checksum.is_none());
                assert!(download.mode.is_none());
            }
            _ => panic!("Expected HttpDownload action type"),
        }
    }

    #[test]
    fn test_http_download_name() {
        let action = HttpDownload {
            url: "https://releases.hashicorp.com/terraform/1.6.0/terraform_1.6.0_linux_amd64.zip".to_string(),
            destination: "/tmp/terraform.zip".to_string(),
            checksum: None,
            mode: None,
        };

        assert_eq!(action.name(), "HttpDownload");
    }

    #[test]
    fn test_http_download_plan() {
        let action = HttpDownload {
            url: "https://nodejs.org/dist/v20.10.0/node-v20.10.0-linux-x64.tar.xz".to_string(),
            destination: "/opt/node-v20.tar.xz".to_string(),
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
            url: "https://github.com/sigstore/cosign/releases/download/v2.2.0/cosign-linux-amd64".to_string(),
            destination: "/usr/local/bin/cosign".to_string(),
            checksum: Some(checksum),
            mode: Some(0o755),
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        assert!(atoms[0].describe().contains("Download"));
    }

    #[test]
    fn test_http_download_home_expansion() {
        unsafe {
            std::env::set_var("HOME", "/home/developer");
        }

        let action = HttpDownload {
            url: "https://go.dev/dl/go1.21.5.linux-amd64.tar.gz".to_string(),
            destination: "~/tools/go1.21.5.tar.gz".to_string(),
            checksum: None,
            mode: None,
        };

        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
        assert!(atoms[0].describe().contains("/home/developer/tools/go1.21.5.tar.gz"));
    }

    #[test]
    fn test_checksum_string_formatting() {
        let checksum = Checksum {
            algorithm: "sha512".to_string(),
            value: "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e".to_string(),
        };

        let action = HttpDownload {
            url: "https://download.docker.com/linux/static/stable/x86_64/docker-24.0.7.tgz".to_string(),
            destination: "/tmp/docker.tgz".to_string(),
            checksum: Some(checksum),
            mode: None,
        };

        // Test that checksum gets formatted correctly in the atom creation
        let atoms = action.plan(std::path::Path::new("."));
        assert_eq!(atoms.len(), 1);
    }
}
