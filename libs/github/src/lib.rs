pub mod types;

use failure::Error;
use hyper_old_types::header::{Link, RelationType};
use log::debug;
use reqwest::{
    header::{AUTHORIZATION, LINK, USER_AGENT},
    Client, Method, RequestBuilder, Response,
};
use std::collections::HashSet;

pub struct GitHub {
    token: String,
    client: Client,
}

impl GitHub {
    pub fn new(token: String) -> Self {
        GitHub {
            token,
            client: Client::new(),
        }
    }

    fn build_request(&self, method: Method, url: &str) -> RequestBuilder {
        let tmp_url;
        let mut url = url.trim_start_matches('/');
        if !url.starts_with("https://") {
            tmp_url = format!("https://api.github.com/{}", url);
            url = &tmp_url;
        }
        debug!("{} {}", method, url);
        self.client
            .request(method, url)
            .header(USER_AGENT, "rust-lang infrastructure tooling")
            .header(AUTHORIZATION, format!("token {}", self.token))
    }

    fn paginated<F>(&self, method: &Method, url: String, mut f: F) -> Result<(), Error>
    where
        F: FnMut(Response) -> Result<(), Error>,
    {
        let mut next = Some(url);
        while let Some(next_url) = next.take() {
            let resp = self
                .build_request(method.clone(), &next_url)
                .send()?
                .error_for_status()?;

            // Extract the next page
            if let Some(links) = resp.headers().get(LINK) {
                let links: Link = links.to_str()?.parse()?;
                for link in links.values() {
                    if link
                        .rel()
                        .map(|r| r.iter().any(|r| *r == RelationType::Next))
                        .unwrap_or(false)
                    {
                        next = Some(link.link().to_string());
                        break;
                    }
                }
            }

            f(resp)?;
        }
        Ok(())
    }

    pub fn get_hooks(&self, repo: &str) -> Result<Vec<types::Hook>, Error> {
        let mut hooks = Vec::new();
        self.paginated(&Method::GET, format!("repos/{}/hooks", repo), |mut resp| {
            let mut content: Vec<types::Hook> = resp.json()?;
            hooks.append(&mut content);
            Ok(())
        })?;
        Ok(hooks)
    }

    pub fn create_hook(&self, repo: &str, hook: &types::HookDetails) -> Result<types::Hook, Error> {
        let res = self
            .build_request(Method::POST, &format!("repos/{}/hooks", repo))
            .json(&hook)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(res)
    }

    pub fn edit_hook_events(
        &self,
        repo: &str,
        id: usize,
        events: &HashSet<types::EventKind>,
    ) -> Result<(), Error> {
        self.build_request(Method::PATCH, &format!("repos/{}/hooks/{}", repo, id))
            .json(&serde_json::json!({
                "events": events,
            }))
            .send()?
            .error_for_status()?;
        Ok(())
    }

    pub fn delete_hook(&self, repo: &str, id: usize) -> Result<(), Error> {
        self.build_request(Method::DELETE, &format!("repos/{}/hooks/{}", repo, id))
            .send()?
            .error_for_status()?;
        Ok(())
    }
}
