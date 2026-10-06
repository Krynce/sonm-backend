use axum::http::{Method, request::Parts};
use sonm_ratelimits::ratelimiter::RatelimitResolver;

pub struct FilesRatelimits;

impl RatelimitResolver<Parts> for FilesRatelimits {
    fn resolve_bucket<'a>(&self, parts: &'a Parts) -> (&'a str, Option<&'a str>) {
        let path = parts
            .uri
            .path()
            .trim_matches('/')
            .split_terminator("/")
            .collect::<Vec<&str>>();

        match (&parts.method, path.as_slice()) {
            (&Method::POST, &[tag]) => ("upload", Some(tag)),
            _ => ("any_atmn", None),
        }
    }

    fn resolve_bucket_limit(&self, bucket: &str) -> u32 {
        match bucket {
            "upload" => 10,
            _ => u32::MAX,
        }
    }
}
