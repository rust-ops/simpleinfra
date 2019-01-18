use failure::{ensure, Error, ResultExt};
use github::types::{self, ContentType, EventKind, InsecureSsl};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Hook {
    pub(crate) url: String,
    pub(crate) content_type: ContentType,
    pub(crate) require_secret: bool,
    pub(crate) events: HashSet<EventKind>,
    #[serde(default)]
    pub(crate) avoid_changes: bool,
}

impl Hook {
    pub(crate) fn to_github(self) -> types::HookDetails {
        types::HookDetails {
            name: "web".to_string(),
            events: self.events,
            active: true,
            config: types::HookConfig {
                url: self.url,
                secret: None,
                content_type: self.content_type,
                insecure_ssl: InsecureSsl::ValidateCerts,
            },
        }
    }

    fn validate(&self) -> Result<(), Error> {
        if self.events.contains(&EventKind::Wildcard) {
            ensure!(
                self.events.len() == 1,
                "an hook with a wildcard can't contain other events"
            );
        }
        Ok(())
    }
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct Repository {
    #[serde(default)]
    pub(crate) presets: Vec<String>,
    #[serde(default)]
    pub(crate) hooks: Vec<Hook>,
}

impl Repository {
    pub(crate) fn hooks<'a>(&'a self, config: &'a Config) -> Vec<&'a Hook> {
        let mut hooks: Vec<_> = self.hooks.iter().collect();
        for preset in &self.presets {
            hooks.push(&config.presets[preset]);
        }
        hooks
    }

    fn validate(&self, presets: &HashMap<String, Hook>) -> Result<(), Error> {
        for preset in &self.presets {
            ensure!(presets.contains_key(preset), "missing preset: {}", preset);
        }
        for hook in &self.hooks {
            hook.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct Config {
    pub(crate) presets: HashMap<String, Hook>,
    pub(crate) repos: HashMap<String, Repository>,
}

impl Config {
    pub(crate) fn load(base: &str) -> Result<Self, Error> {
        let base = PathBuf::from(base);
        ensure!(
            base.is_dir(),
            "config directory {} doesn't exist",
            base.display()
        );

        let mut presets = HashMap::new();
        let mut repos = HashMap::new();

        for entry in std::fs::read_dir(&base.join("presets"))? {
            let path = entry?.path();
            if let (Some(ext), Some(file_stem)) = (path.extension(), path.file_stem()) {
                if ext.to_str() == Some("toml") {
                    let name = file_stem.to_str().unwrap().into();
                    let preset: Hook = toml::from_str(&std::fs::read_to_string(&path)?)
                        .with_context(|_| format!("failed to parse preset `{}`", name))?;
                    preset
                        .validate()
                        .with_context(|_| format!("failed to validate preset `{}`", name))?;
                    presets.insert(name, preset);
                }
            }
        }

        for org_entry in std::fs::read_dir(&base.join("repos"))? {
            let org_entry = org_entry?;
            for repo_entry in std::fs::read_dir(&org_entry.path())? {
                let path = repo_entry?.path();
                if let (Some(ext), Some(file_stem)) = (path.extension(), path.file_stem()) {
                    if ext.to_str() == Some("toml") {
                        let name = format!(
                            "{}/{}",
                            org_entry.file_name().to_str().unwrap(),
                            file_stem.to_str().unwrap()
                        );
                        let repo: Repository = toml::from_str(&std::fs::read_to_string(&path)?)
                            .with_context(|_| format!("failed to parse repo `{}`", name))?;
                        repo.validate(&presets)
                            .with_context(|_| format!("failed to validate repo `{}`", name))?;
                        repos.insert(name, repo);
                    }
                }
            }
        }

        Ok(Config { presets, repos })
    }
}
