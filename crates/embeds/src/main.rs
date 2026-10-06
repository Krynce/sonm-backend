use std::net::{Ipv4Addr, SocketAddr};

use axum::Router;

use tokio::net::TcpListener;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};
use utoipa_scalar::{Scalar, Servable as ScalarServable};

mod api;
pub mod requests;
pub mod specialty;
pub mod website_embed;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Configure logging and environment
    sonm_config::configure!(embeds);

    // Configure API schema
    #[derive(OpenApi)]
    #[openapi(
        paths(api::root, api::proxy, api::embed),
        components(schemas(
            api::RootResponse,
            sonm_result::Error,
            sonm_result::ErrorType,
            sonm_models::v0::ImageSize,
            sonm_models::v0::Image,
            sonm_models::v0::Video,
            sonm_models::v0::TwitchType,
            sonm_models::v0::LightspeedType,
            sonm_models::v0::BandcampType,
            sonm_models::v0::Special,
            sonm_models::v0::WebsiteMetadata,
            sonm_models::v0::Text,
            sonm_models::v0::Embed
        ))
    )]
    struct ApiDoc;

    #[allow(dead_code)]
    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "api_key",
                    SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
                )
            }
        }
    }

    // Configure Axum and router
    let app = Router::new()
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
        .nest("/", api::router().await);

    // Configure TCP listener and bind
    tracing::info!("Listening on 0.0.0.0:14705");
    tracing::info!("Play around with the API: http://localhost:14705/scalar");
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 14705));
    let listener = TcpListener::bind(&address).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
}
