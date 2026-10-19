use serde::Deserialize;
use sonm_models::v0::{Embed, Special, WebsiteMetadata};
use sonm_result::{Result, create_error};

use crate::requests::{RE_URL_YOUTUBE, Request};
pub struct SpecialtySitesGenerator {}

#[derive(Debug, Deserialize)]
struct YoutubeOEmbed {
    pub title: String,
    pub author_name: String,
    pub author_url: String,
    pub provider_name: String,
    pub thumbnail_url: String,
}

impl SpecialtySitesGenerator {
    pub async fn youtube(url: &str, request: Request) -> Result<Embed> {
        let json: YoutubeOEmbed = request
            .response
            .json()
            .await
            .map_err(|_| create_error!(ProxyError))?;

        let captures = RE_URL_YOUTUBE
            .captures(url)
            .ok_or_else(|| create_error!(ProxyError))?;
        let id = captures[1].to_string();
        let timestamp = captures
            .get(2)
            .map(|e| Some(e.as_str().to_string()))
            .unwrap_or(None);

        Ok(Embed::Website(WebsiteMetadata {
            url: Some(url.to_string()),
            original_url: None,
            special: Some(Special::YouTube {
                id,
                timestamp,
                creator_name: Some(json.author_name),
                creator_url: Some(json.author_url),
            }),
            title: Some(json.title),
            description: None,
            image: None,
            video: None,
            site_name: Some(json.provider_name),
            icon_url: Some(json.thumbnail_url),
            colour: None,
        }))
    }
}
