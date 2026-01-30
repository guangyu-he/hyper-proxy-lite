use anyhow::Result;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Response, StatusCode};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub enum FilterMode {
    Blacklist,
    Whitelist,
}

#[derive(Deserialize)]
pub struct FilterRules {
    mode: FilterMode,
    domains: HashSet<String>,
}

impl FilterRules {
    #[allow(dead_code)]
    pub fn new_blacklist<S: Into<String>>(domains: Vec<S>) -> Self {
        FilterRules {
            mode: FilterMode::Blacklist,
            domains: domains.into_iter().map(|s| s.into()).collect(),
        }
    }

    #[allow(dead_code)]
    pub fn new_whitelist<S: Into<String>>(domains: Vec<S>) -> Self {
        FilterRules {
            mode: FilterMode::Whitelist,
            domains: domains.into_iter().map(|s| s.into()).collect(),
        }
    }

    /// Check if a given host is allowed based on the filter rules.
    /// This function extracts the domain from the host (ignoring port)
    /// and checks it against the filter mode and domain list.
    pub fn is_allowed(&self, host: &str) -> bool {
        let domain = host.split(':').next().unwrap_or(host).to_string();

        match self.mode {
            FilterMode::Blacklist => !self.domains.contains(&domain),
            FilterMode::Whitelist => self.domains.contains(&domain),
        }
    }

    /// Read filter rules from a TOML configuration file.
    /// The configuration file should specify the filter mode
    /// (blacklist or whitelist) and the list of domains.
    pub fn read_config_file<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let path = path.into();
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "Filter config file does not exist: {}",
                path.display()
            ));
        }
        let content = std::fs::read_to_string(&path).map_err(|e| {
            anyhow::anyhow!(
                "Failed to read filter config file {}: {}",
                path.display(),
                e
            )
        })?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;
        Ok(config)
    }
}

/// Generate a blocked response for a given host.
/// This function creates an HTTP 403 Forbidden response
/// with a message indicating that access to the specified host
/// is blocked by proxy filter rules.
pub fn blocked_response(host: &str) -> Result<Response<BoxBody<Bytes, hyper::Error>>> {
    let body = format!("Access to {} is blocked by proxy filter rules", host);

    let response = Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header("Content-Type", "text/plain")
        .body(
            Full::new(Bytes::from(body))
                .map_err(|never| match never {})
                .boxed(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to build blocked response: {}", e))?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use crate::addon::filter::FilterRules;

    #[test]
    fn test_read_file() -> anyhow::Result<()> {
        let rules = FilterRules::read_config_file(
            "/Users/guangyu/RustroverProjects/hyper-proxy-lite/filter_rules_example.toml",
        )?;
        assert_eq!(rules.mode, crate::addon::filter::FilterMode::Blacklist);
        assert!(rules.domains.contains("example.com"));
        Ok(())
    }
}
