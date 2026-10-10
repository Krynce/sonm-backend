use encoding_rs::{Encoding, UTF_8_INIT};
use lazy_static::lazy_static;
use mime::Mime;
use pdk_ip_filter_lib::IpFilter;
use regex::Regex;
use reqwest::{
    Client, Response,
    dns::{Addrs, Name, Resolve},
    header::{self, CONTENT_TYPE},
    redirect,
};
use sonm_config::{config, report_internal_error};
use sonm_models::v0::{Embed, Image, ImageSize, Video};
use sonm_result::{Error, Result, ToSonmError, create_error};
use sonm_storage::{create_thumbnail, decode_image, image_size_vec, is_valid_image, video_size};
use std::net::{IpAddr, SocketAddr};
use std::{
    io::{Cursor, Write},
    str::FromStr,
    time::Duration,
};
use url::{Host, Url};

use crate::specialty;

lazy_static! {
    /// Request client
    static ref CLIENT: Client = reqwest::Client::builder()
        .dns_resolver(CachedDnsResolver {})
        .timeout(Duration::from_secs(10)) // TODO config
        .connect_timeout(Duration::from_secs(5)) // TODO config
        .redirect(redirect::Policy::none())
        .build()
        .expect("reqwest Client");

    /// Spoof User Agent as Discord
    static ref RE_USER_AGENT_SPOOFING_AS_DISCORD: Regex = Regex::new("^(?:(?:vx|fx)?twitter|(?:fixv|fixup)?x|(?:old\\.|new\\.|www\\.)reddit)\\.com|klipy\\.com").expect("valid regex");

    /// Regex for matching new Reddit URLs
    static ref RE_URL_NEW_REDDIT: Regex = Regex::new("^(?:(?:new\\.|www\\.)?reddit).com").expect("valid regex");

    /// Regex for matching YouTube Shorts URLs
    pub static ref RE_URL_YOUTUBE_SHORTS: Regex = Regex::new("^(?:(?:https?:)?//)?(?:(?:www\\.)?youtube\\.com)/shorts/([a-zA-Z0-9_-]+)").expect("valid regex");

    /// Regex for matching YouTube URLs
    pub static ref RE_URL_YOUTUBE: Regex = Regex::new("^(?:(?:https?:)?//)?(?:(?:www|m)\\.)?(?:(?:youtube\\.com|youtu\\.be))(?:/(?:[\\w\\-]+\\?v=|embed/|v/|shorts/)?)([\\w\\-]+)(?:(?:&t|&start)=([\\d]+))?(?:\\S+)?$").unwrap();

    /// Url for YouTube oembed
    pub static ref OEMBED_URL: Url = Url::parse("https://www.youtube.com/oembed").unwrap();

    /// Cache for embed results
    static ref EMBED_CACHE: moka::future::Cache<String, Embed> = moka::future::Cache::builder()
        // TODO config
        .max_capacity(10_000) // Cache up to 10k embeds
        .time_to_live(Duration::from_secs(60)) // For up to 1 minute
        .build();

    static ref DNS_CACHE: moka::future::Cache<String, Vec<SocketAddr>> = moka::future::Cache::builder()
        .max_capacity(10_000)
        .time_to_idle(Duration::from_secs(30))
        .build();

    static ref IP_BLOCKLIST: IpFilter = IpFilter::block(&[
        "0.0.0.0/8",
        "10.0.0.0/8",
        "192.168.0.0/16",
        "127.0.0.0/8",
        "172.16.0.0/12",
        "169.254.0.0/16",   // link-local, incl. the cloud metadata address
        "100.64.0.0/10",    // CGNAT
        "192.0.0.0/24",     // IETF protocol assignments
        "198.18.0.0/15",    // benchmarking
        "224.0.0.0/4",      // multicast
        "::1",
        "::",
        "fc00::/7",         // unique local, covers fc00::/10 and fd00::/8
        "fe80::/10",        // link-local
        "ff00::/8",         // multicast
        ]
    ).unwrap();
}

/// Check a resolved address against [`IP_BLOCKLIST`]
///
/// IPv4-mapped IPv6 addresses are rejected outright: the filter would not match them against the
/// IPv4 ranges above.
fn ip_is_allowed(ip: &IpAddr) -> bool {
    let string = ip.to_string();
    !string.contains("::ffff:") && IP_BLOCKLIST.is_allowed(&string)
}

/// Maximum size of a response body this service will pull into memory
///
/// ponytail: one number for every kind of media; split per mime type if it ever matters.
const MAX_RESPONSE_SIZE: usize = 20 * 1024 * 1024;

/// Read a response body, refusing anything larger than [`MAX_RESPONSE_SIZE`]
///
/// `Content-Length` is only a hint, so the chunks are counted as well.
async fn read_body(mut response: Response) -> Result<Vec<u8>> {
    let too_large = || {
        create_error!(FileTooLarge {
            max: MAX_RESPONSE_SIZE
        })
    };

    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_SIZE as u64)
    {
        return Err(too_large());
    }

    let mut body = Vec::new();

    while let Some(chunk) = report_internal_error!(response.chunk().await)? {
        if body.len() + chunk.len() > MAX_RESPONSE_SIZE {
            return Err(too_large());
        }

        body.extend_from_slice(&chunk);
    }

    Ok(body)
}

static PROXY_CACHE: tokio::sync::OnceCell<moka::future::Cache<String, Result<(String, Vec<u8>)>>> =
    tokio::sync::OnceCell::const_new();

/// Cache for proxy results
async fn proxy_cache() -> &'static moka::future::Cache<String, Result<(String, Vec<u8>)>> {
    PROXY_CACHE
        .get_or_init(|| async {
            moka::future::Cache::builder()
                .weigher(|_key, value: &Result<(String, Vec<u8>)>| -> u32 {
                    std::mem::size_of::<Result<(String, Vec<u8>)>>() as u32
                        + if let Ok((url, vec)) = value {
                            url.len().try_into().unwrap_or(u32::MAX)
                                + vec.len().try_into().unwrap_or(u32::MAX)
                        } else {
                            std::mem::size_of::<Error>() as u32
                        }
                })
                .max_capacity(config().await.embeds.proxy_cache_size)
                .time_to_live(Duration::from_secs(60)) // For up to 1 minute
                .build()
        })
        .await
}

struct CachedDnsResolver {}

impl reqwest::dns::Resolve for CachedDnsResolver {
    fn resolve(&self, name: Name) -> reqwest::dns::Resolving {
        Box::pin(async move {
            {
                if let Some(addrs) = DNS_CACHE.get(&name.as_str().to_string()).await {
                    let resp: Addrs = Box::new(addrs.clone().into_iter());
                    return Ok(resp);
                }
            }

            let mut lookup = name.as_str().to_string();
            if !lookup.contains(":") {
                lookup += ":0";
            }

            let fallback: Vec<SocketAddr> = tokio::net::lookup_host(&lookup)
                .await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?
                .collect();

            // Filtering here rather than only before the request is what closes DNS rebinding:
            // this is the resolution the connection actually uses, redirects included.
            if let Some(blocked) = fallback.iter().find(|addr| !ip_is_allowed(&addr.ip())) {
                return Err(format!(
                    "{} resolves to blocked address {}",
                    name.as_str(),
                    blocked.ip()
                )
                .into());
            }

            {
                DNS_CACHE
                    .insert(name.as_str().to_string().clone(), fallback.clone())
                    .await;
                let addrs: Addrs = Box::new(fallback.clone().into_iter());
                Ok(addrs)
            }
        })
    }
}

/// Information about a successful request
pub struct Request {
    pub response: Response,
    pub mime: Mime,
}

impl Request {
    /// Proxy a given URL
    pub async fn proxy_file(url: &str) -> Result<(String, Vec<u8>)> {
        let cache = proxy_cache().await;

        if let Some(hit) = cache.get(url).await {
            hit
        } else {
            let Request { response, mime } = Request::new_from_str(url).await?;

            if matches!(mime.type_(), mime::IMAGE | mime::VIDEO) {
                let bytes = read_body(response).await;

                let result = match bytes {
                    Ok(bytes) => {
                        if matches!(mime.type_(), mime::IMAGE) {
                            let reader = &mut Cursor::new(&bytes);

                            if matches!(mime.subtype(), mime::GIF) {
                                if is_valid_image(reader, "image/gif") {
                                    Ok(("image/gif".to_owned(), bytes.to_vec()))
                                } else {
                                    Err(create_error!(FileTypeNotAllowed))
                                }
                            } else {
                                Ok((
                                    "image/webp".to_owned(),
                                    create_thumbnail(
                                        decode_image(reader, mime.as_ref())?,
                                        "attachments",
                                    )
                                    .await,
                                ))
                            }
                        } else {
                            let mut file = report_internal_error!(tempfile::NamedTempFile::new())?;
                            report_internal_error!(file.write_all(&bytes))?;

                            if video_size(&file).is_some() {
                                Ok((mime.to_string(), bytes.to_vec()))
                            } else {
                                Err(create_error!(FileTypeNotAllowed))
                            }
                        }
                    }
                    Err(err) => Err(err),
                };

                cache.insert(url.to_owned(), result.clone()).await;
                result
            } else {
                Err(create_error!(FileTypeNotAllowed))
            }
        }
    }

    /// Fetch metadata for an image
    pub async fn fetch_image_metadata(
        url: &str,
        request: Option<Request>,
        size: ImageSize,
    ) -> Result<Option<Image>> {
        if let Some(hit) = EMBED_CACHE.get(url).await {
            match hit {
                Embed::Image(img) => Ok(Some(img)),
                _ => Ok(None),
            }
        } else {
            let request = if let Some(request) = request {
                request
            } else {
                let request = Request::new_from_str(url).await?;
                if matches!(request.mime.type_(), mime::IMAGE) {
                    request
                } else {
                    return Err(create_error!(FileTypeNotAllowed));
                }
            };

            if let Some((width, height)) =
                image_size_vec(&read_body(request.response).await?, request.mime.as_ref())
            {
                Ok(Some(Image {
                    url: url.to_owned(),
                    width,
                    height,
                    size,
                }))
            } else {
                Ok(None)
            }
        }
    }

    /// Fetch metadata for an video
    pub async fn fetch_video_metadata(
        url: &str,
        request: Option<Request>,
    ) -> Result<Option<Video>> {
        if let Some(hit) = EMBED_CACHE.get(url).await {
            match hit {
                Embed::Video(vid) => Ok(Some(vid)),
                _ => Ok(None),
            }
        } else {
            let response = if let Some(Request { response, .. }) = request {
                response
            } else {
                let Request { response, mime } = Request::new_from_str(url).await?;
                if matches!(mime.type_(), mime::VIDEO) {
                    response
                } else {
                    return Err(create_error!(FileTypeNotAllowed));
                }
            };

            let mut file = report_internal_error!(tempfile::NamedTempFile::new())?;
            report_internal_error!(file.write_all(&read_body(response).await?))?;

            if let Some((width, height)) = video_size(&file) {
                Ok(Some(Video {
                    url: url.to_owned(),
                    width: width as usize,
                    height: height as usize,
                }))
            } else {
                Ok(None)
            }
        }
    }

    /// Generate embed for a given URL
    pub async fn generate_embed(mut url: String) -> Result<Embed> {
        // Re-map certain links for better metadata generation
        if RE_URL_NEW_REDDIT.is_match(&url) {
            url = RE_URL_NEW_REDDIT
                // Reddit has a bunch of clickbait-y marketing on the new URLs, so we use the old site instead
                .replace(&url, "https://old.reddit.com")
                .to_string();
        }

        // Re-map Youtube Shorts to regular Youtube links
        if let Some(captures) = RE_URL_YOUTUBE_SHORTS.captures(&url) {
            if let Some(video_id) = captures.get(1) {
                url = format!("https://youtube.com/watch?v={}", video_id.as_str());
            }
        }

        // Generate the actual embed
        if let Some(hit) = EMBED_CACHE.get(&url).await {
            Ok(hit)
        } else if RE_URL_YOUTUBE.is_match(&url) {
            let mut yt_url = OEMBED_URL.clone();
            yt_url.set_query(Some(&format!("url={url}")));

            let request = Request::new(yt_url).await?;
            let embed = specialty::SpecialtySitesGenerator::youtube(&url, request).await?;

            EMBED_CACHE.insert(url.to_owned(), embed.clone()).await;

            Ok(embed)
        } else {
            let request = Request::new_from_str(&url).await?;
            let embed = match (request.mime.type_(), request.mime.subtype()) {
                (_, mime::HTML) => {
                    let content_type = request
                        .response
                        .headers()
                        .get(header::CONTENT_TYPE)
                        .and_then(|value| value.to_str().ok())
                        .and_then(|value| value.parse::<Mime>().ok());

                    let encoding_name = content_type
                        .as_ref()
                        .and_then(|mime| mime.get_param("charset").map(|charset| charset.as_str()))
                        .unwrap_or("utf-8");

                    let encoding =
                        Encoding::for_label(encoding_name.as_bytes()).unwrap_or(&UTF_8_INIT);

                    let bytes = read_body(request.response).await?;
                    let (text, _, _) = encoding.decode(&bytes);

                    crate::website_embed::create_website_embed(&url, &text)
                        .await
                        .map(Embed::Website)
                        .unwrap_or_default()
                }
                (mime::IMAGE, _) => {
                    Request::fetch_image_metadata(&url, Some(request), ImageSize::Large)
                        .await
                        .map(|res| res.map(Embed::Image).unwrap_or_default())
                        .unwrap_or_default()
                }
                (mime::VIDEO, _) => Request::fetch_video_metadata(&url, Some(request))
                    .await
                    .map(|res| res.map(Embed::Video).unwrap_or_default())
                    .unwrap_or_default(),
                _ => Embed::None,
            };

            EMBED_CACHE.insert(url.to_owned(), embed.clone()).await;
            Ok(embed)
        }
    }

    /// Send a new request to a service
    pub async fn new(url: Url) -> Result<Request> {
        let mut url = url;
        let url_host_str = url.host_str().ok_or(create_error!(ProxyError))?.to_string();

        Request::ensure_url_allowed(&url).await?;

        let mut redirect_count = 0;

        loop {
            let response = CLIENT
            .get(url)
            .header(
                "User-Agent",
                if RE_USER_AGENT_SPOOFING_AS_DISCORD.is_match(&url_host_str) {
                    "Mozilla/5.0 (compatible; Discordbot/2.0; +https://discordapp.com)"
                } else {
                    "Mozilla/5.0 (compatible; Sonm-Embeds/1.0; +https://github.com/krynce/sonm-backend)"
                },
            )
            .header("Accept-Language", "en-US,en;q=0.5")
            .send()
            .await
            .map_err(|_| create_error!(ProxyError))?;

            if response.status().is_redirection() {
                redirect_count += 1;

                if redirect_count > 5 {
                    return Err(create_error!(ProxyError));
                }
                if let Some(location) = response.headers().get("location") {
                    let location = location.to_str().map_err(|_| create_error!(ProxyError))?;
                    url = Url::from_str(location).to_internal_error()?;

                    Request::ensure_url_allowed(&url).await?;

                    continue;
                } else {
                    return Err(create_error!(ProxyError));
                }
            }

            if !response.status().is_success() {
                tracing::error!("{:?}", response);
                return Err(create_error!(ProxyError));
            }

            let content_type = response
                .headers()
                .get(CONTENT_TYPE)
                .ok_or(create_error!(ProxyError))?
                .to_str()
                .map_err(|_| create_error!(ProxyError))?;

            let mime: mime::Mime = content_type
                .parse()
                .map_err(|_| create_error!(ProxyError))?;

            return Ok(Request { response, mime });
        }
    }

    pub async fn new_from_str(url: &str) -> Result<Request> {
        let proper_url = Url::parse(url).map_err(|_| create_error!(ProxyError))?;
        Request::new(proper_url).await
    }

    /// Refuse URLs pointing at anything but a public address
    ///
    /// For domains this is only the first gate; the resolution the connection actually uses is
    /// filtered in [`CachedDnsResolver`].
    pub async fn ensure_url_allowed(url: &Url) -> Result<()> {
        match url.host() {
            Some(Host::Ipv4(ipv4)) => {
                if !ip_is_allowed(&ipv4.into()) {
                    return Err(create_error!(InvalidOperation));
                }
            }
            Some(Host::Ipv6(ipv6)) => {
                if !ip_is_allowed(&ipv6.into()) {
                    return Err(create_error!(InvalidOperation));
                }
            }
            Some(Host::Domain(domain)) => {
                let config = config().await;

                if !domain.contains(".") // lazily block TLDs
                    || config.embeds.blocked_domains.iter().any(|x| x == domain)
                {
                    return Err(create_error!(InvalidOperation));
                }

                // Resolve once up front so a blocked host fails here rather than mid-request;
                // the resolver caches and filters, so the connection cannot land elsewhere.
                let name = Name::from_str(domain).map_err(|_| create_error!(ProxyError))?;

                // The addresses themselves are not needed here, only that the lookup succeeded
                // and passed the filter; the connection resolves through the same cache.
                let _ = CachedDnsResolver {}
                    .resolve(name)
                    .await
                    .map_err(|_| create_error!(ProxyError))?;
            }
            None => return Err(create_error!(ProxyError)),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_RESPONSE_SIZE, ip_is_allowed, read_body};
    use std::net::IpAddr;
    use std::str::FromStr;

    #[test]
    fn blocks_everything_that_is_not_public() {
        for blocked in [
            "127.0.0.1",
            "10.1.2.3",
            "192.168.1.1",
            "172.16.0.1",
            "169.254.169.254", // cloud metadata
            "100.64.0.1",      // CGNAT
            "192.0.0.1",
            "198.18.0.1",
            "224.0.0.1",
            "0.0.0.0",
            "::1",
            "::",
            "fd00::1",
            "fe80::1",
            "ff02::1",
            "::ffff:127.0.0.1",
        ] {
            assert!(
                !ip_is_allowed(&IpAddr::from_str(blocked).unwrap()),
                "{blocked} should be blocked"
            );
        }

        for allowed in ["1.1.1.1", "93.184.216.34", "2606:4700:4700::1111"] {
            assert!(
                ip_is_allowed(&IpAddr::from_str(allowed).unwrap()),
                "{allowed} should be allowed"
            );
        }
    }

    fn response(body: Vec<u8>) -> reqwest::Response {
        reqwest::Response::from(axum::http::Response::new(body))
    }

    #[tokio::test]
    async fn reads_a_small_body() {
        assert_eq!(
            read_body(response(b"hello".to_vec())).await.unwrap(),
            b"hello"
        );
    }

    #[tokio::test]
    async fn refuses_an_oversized_body() {
        assert!(
            read_body(response(vec![0; MAX_RESPONSE_SIZE + 1]))
                .await
                .is_err()
        );
    }
}
