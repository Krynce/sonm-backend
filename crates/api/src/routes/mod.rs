use revolt_rocket_okapi::{revolt_okapi::openapi3::OpenApi, settings::OpenApiSettings};
pub use rocket::http::Status;
pub use rocket::response::Redirect;
use rocket::{Build, Rocket};
use sonm_config::Settings;

mod account;
mod bots;
mod channels;
mod customisation;
mod invites;
mod mfa;
mod onboard;
mod openapi;
mod policy;
mod push;
mod root;
mod safety;
mod servers;
mod session;
mod sync;
mod users;
mod webhooks;

pub fn mount(config: Settings, mut rocket: Rocket<Build>) -> Rocket<Build> {
    let settings = OpenApiSettings::default();

    let webhooks = if config.features.webhooks_enabled {
        webhooks::routes()
    } else {
        (vec![], OpenApi::new())
    };

    mount_endpoints_and_merged_docs! {
        rocket, "/".to_owned(), settings,
        "/" => (vec![], openapi::custom_openapi_spec()),
        "" => openapi_get_routes_spec![root::root],
        "/users" => users::routes(),
        "/bots" => bots::routes(),
        "/channels" => channels::routes(),
        "/servers" => servers::routes(),
        "/invites" => invites::routes(),
        "/custom" => customisation::routes(),
        "/safety" => safety::routes(),
        "/auth/account" => account::routes(),
        "/auth/session" => session::routes(),
        "/auth/mfa" => mfa::routes(),
        "/onboard" => onboard::routes(),
        "/policy" => policy::routes(),
        "/push" => push::routes(),
        "/sync" => sync::routes(),
        "/webhooks" => webhooks
    };

    rocket
}
