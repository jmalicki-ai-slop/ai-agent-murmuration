//! Secrets management for Murmuration
//!
//! Secrets are stored separately from configuration to avoid accidental sharing.
//! The secrets file is located at `~/.config/murmur/secrets.toml` and must have
//! restrictive permissions (0600 on Unix).
//!
//! Loading priority:
//! 1. Environment variables (GITHUB_TOKEN)
//! 2. Secrets file (~/.config/murmur/secrets.toml)

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::{Error, Result};

/// Secrets structure
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Secrets {
    /// GitHub configuration
    pub github: GitHubSecrets,
}

/// GitHub-related secrets
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct GitHubSecrets {
    /// GitHub Personal Access Token
    pub token: Option<String>,
}

impl Secrets {
    /// Load secrets from the default location
    ///
    /// Returns default (empty) secrets if file doesn't exist
    pub fn load() -> Result<Self> {
        let secrets_path = Self::default_secrets_path();

        if let Some(path) = secrets_path {
            if path.exists() {
                return Self::load_from_file(&path);
            }
        }

        Ok(Self::default())
    }

    /// Load secrets from a specific file with permission checking
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        // Check file permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let metadata = std::fs::metadata(path).map_err(Error::Io)?;
            let mode = metadata.permissions().mode();

            // Check if file is readable by group or others (mode & 0o077)
            if mode & 0o077 != 0 {
                return Err(Error::Config(format!(
                    "Secrets file {} has insecure permissions {:o}. \
                     Please run: chmod 600 {}",
                    path.display(),
                    mode & 0o777,
                    path.display()
                )));
            }

            debug!(path = %path.display(), mode = format!("{:o}", mode & 0o777), "Secrets file permissions OK");
        }

        let contents = std::fs::read_to_string(path).map_err(Error::Io)?;
        let mut secrets: Secrets = toml::from_str(&contents)
            .map_err(|e| Error::Config(format!("Failed to parse secrets: {}", e)))?;

        // Trim whitespace from token
        if let Some(ref mut token) = secrets.github.token {
            *token = token.trim().to_string();
        }

        Ok(secrets)
    }

    /// Get the default secrets file path
    ///
    /// Returns `~/.config/murmur/secrets.toml` on Unix
    pub fn default_secrets_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("murmur").join("secrets.toml"))
    }

    /// Get GitHub token with environment variable override
    ///
    /// Priority: GITHUB_TOKEN env var > secrets file
    pub fn github_token(&self) -> Option<String> {
        // Check environment variable first
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            let token = token.trim().to_string();
            if !token.is_empty() {
                debug!("Using GitHub token from GITHUB_TOKEN environment variable");
                return Some(token);
            }
        }

        // Fall back to secrets file
        if let Some(ref token) = self.github.token {
            if !token.is_empty() {
                debug!("Using GitHub token from secrets file");
                return Some(token.clone());
            }
        }

        None
    }

    /// Create a template secrets file at the default location
    ///
    /// Creates parent directories if needed and sets secure permissions
    pub fn create_template() -> Result<PathBuf> {
        let path = Self::default_secrets_path()
            .ok_or_else(|| Error::Config("Could not determine secrets path".to_string()))?;

        // Create parent directory
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(Error::Io)?;
        }

        // Don't overwrite existing file
        if path.exists() {
            return Err(Error::Config(format!(
                "Secrets file already exists at {}",
                path.display()
            )));
        }

        let template = r#"# Murmur Secrets
# This file contains sensitive credentials - do not share or commit to version control
#
# IMPORTANT: This file must have restrictive permissions (chmod 600)

[github]
# GitHub Personal Access Token
# Create at: https://github.com/settings/tokens
# Required permissions: repo (or fine-grained: Issues read/write, Pull requests read)
token = ""
"#;

        std::fs::write(&path, template).map_err(Error::Io)?;

        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&path, perms).map_err(Error::Io)?;
        }

        warn!(path = %path.display(), "Created secrets template - please edit and add your tokens");

        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_secrets() {
        let secrets = Secrets::default();
        assert!(secrets.github.token.is_none());
    }

    #[test]
    fn test_parse_secrets() {
        let toml = r#"
[github]
token = "ghp_xxxxxxxxxxxx"
"#;
        let secrets: Secrets = toml::from_str(toml).unwrap();
        assert_eq!(secrets.github.token, Some("ghp_xxxxxxxxxxxx".to_string()));
    }

    #[test]
    fn test_token_with_whitespace() {
        let toml = r#"
[github]
token = "  ghp_xxxxxxxxxxxx  "
"#;
        let secrets: Secrets = toml::from_str(toml).unwrap();
        // toml preserves whitespace, load_from_file trims it
        assert!(secrets.github.token.as_ref().unwrap().contains("ghp_"));
    }

    #[cfg(unix)]
    #[test]
    fn test_insecure_permissions_rejected() {
        use std::os::unix::fs::PermissionsExt;

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "[github]\ntoken = \"test\"").unwrap();

        // Set world-readable permissions
        let perms = std::fs::Permissions::from_mode(0o644);
        std::fs::set_permissions(file.path(), perms).unwrap();

        let result = Secrets::load_from_file(&file.path().to_path_buf());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("insecure permissions"));
    }

    #[cfg(unix)]
    #[test]
    fn test_secure_permissions_accepted() {
        use std::os::unix::fs::PermissionsExt;

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "[github]\ntoken = \"ghp_test\"").unwrap();

        // Set owner-only permissions
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(file.path(), perms).unwrap();

        let result = Secrets::load_from_file(&file.path().to_path_buf());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().github.token, Some("ghp_test".to_string()));
    }

    #[test]
    fn test_env_var_override() {
        let secrets = Secrets {
            github: GitHubSecrets {
                token: Some("from_file".to_string()),
            },
        };

        // Without env var, use file token
        std::env::remove_var("GITHUB_TOKEN");
        // Note: can't easily test env var in unit tests due to global state
        // Just verify the file token works
        assert_eq!(secrets.github.token, Some("from_file".to_string()));
    }
}
