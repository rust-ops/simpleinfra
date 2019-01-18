mod config;

use crate::config::Config;
use failure::{bail, Error};
use log::{error, info, warn};
use std::collections::HashMap;
use structopt::StructOpt;

static DEFAULT_CONFIG_DIR: &str = "config/repo-webhooks";
static APPVEYOR_BASE: &str = "https://ci.appveyor.com/api/github/webhook?id=";

#[derive(StructOpt)]
struct Cli {
    #[structopt(help = "path to the configuration directory")]
    config: Option<String>,
    #[structopt(help = "avoid making changes on GitHub", long = "dry")]
    dry: bool,
}

fn normalize(mut details: github::types::HookDetails) -> github::types::HookDetails {
    // Damn AppVeyor and the secrets in the URLs
    if details.config.url.starts_with(APPVEYOR_BASE) {
        details.config.url = format!("{}{}", APPVEYOR_BASE, "{secret}");
    }
    details
}

fn webhook_url(name: &str, id: usize) -> String {
    format!("https://github.com/{}/settings/hooks/{}", name, id)
}

fn app() -> Result<(), Error> {
    let cli = Cli::from_args();
    let config = Config::load(
        cli.config
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or(DEFAULT_CONFIG_DIR),
    )?;
    let github = if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        github::GitHub::new(token)
    } else {
        bail!("missing environment variable GITHUB_TOKEN");
    };

    if cli.dry {
        info!("this is a dry run, no changes will be applied");
    }

    for (name, repo) in &config.repos {
        let mut expected: HashMap<_, _> = repo
            .hooks(&config)
            .into_iter()
            .map(|h| (h.url.clone(), (normalize((*h).clone().to_github()), h)))
            .collect();
        let actual = github
            .get_hooks(&name)?
            .into_iter()
            .map(|mut hook| {
                hook.details = normalize(hook.details);
                hook
            })
            .collect::<Vec<_>>();

        for hook in &actual {
            if let Some((expected, expected_config)) = expected.remove(&hook.details.config.url) {
                if expected != hook.details {
                    if expected_config.avoid_changes {
                        warn!(
                            "{}: needs updates, but marked as avoid-changes: {}",
                            name, expected.config.url
                        );
                    } else {
                        if expected.events != hook.details.events {
                            info!("{}: updating webhook events: {}", name, expected.config.url);
                            if !cli.dry {
                                github.edit_hook_events(&name, hook.id, &expected.events)?;
                            }
                        }
                        if expected.config != hook.details.config {
                            warn!(
                                "{}: update configuration: {}",
                                name,
                                webhook_url(&name, hook.id)
                            );
                        }
                    }
                }
                if hook.details.config.secret.is_none() && expected_config.require_secret {
                    warn!("{}: set secret key: {}", name, webhook_url(&name, hook.id),);
                } else if hook.details.config.secret.is_some() && !expected_config.require_secret {
                    warn!(
                        "{}: remove secret key: {}",
                        name,
                        webhook_url(&name, hook.id),
                    );
                }
            } else {
                info!("{}: removing webhook: {}", name, hook.details.config.url);
                if !cli.dry {
                    github.delete_hook(&name, hook.id)?;
                }
            }
        }

        for (_, (new, expected_config)) in expected.into_iter() {
            if expected_config.avoid_changes {
                warn!(
                    "{}: needs to be created, but marked as avoid-changes: {}",
                    name, new.config.url
                );
            } else {
                info!("{}: creating webhook: {}", name, new.config.url);
                if !cli.dry {
                    let new_hook = github.create_hook(&name, &new)?;
                    if expected_config.require_secret {
                        warn!(
                            "{}: set secret key: {}",
                            name,
                            webhook_url(&name, new_hook.id),
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() {
    let mut logger = env_logger::Builder::new();
    logger.filter_module("sync_webhooks", log::LevelFilter::Info);
    if let Ok(content) = std::env::var("RUST_LOG") {
        logger.parse(&content);
    }
    logger.init();

    if let Err(err) = app() {
        error!("{}", err);
        for cause in err.iter_causes() {
            error!("caused by: {}", cause);
        }
    }
}
