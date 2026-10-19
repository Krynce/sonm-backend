use reqwest::Client;
use sha1::Digest;
use std::{collections::HashSet, sync::LazyLock};

use sonm_config::config;
use sonm_result::{Result, ToSonmError};

static CLIENT: LazyLock<Client> = LazyLock::new(Client::new);
static ARGON_CONFIG: LazyLock<argon2::Config<'static>> = LazyLock::new(argon2::Config::default);
static TOP_100K_COMPROMISED: LazyLock<HashSet<String>> = LazyLock::new(|| {
    include_str!("../../assets/pwned100k.txt")
        .split('\n')
        .map(|x| x.into())
        .collect()
});

#[derive(Deserialize)]
struct EasyPwnedResult {
    secure: bool,
}

/// Hash a password using argon2
///
/// argon2 is deliberately expensive — tens of ms of CPU — so it never runs on a tokio
/// worker thread.
pub async fn hash_password(plaintext_password: String) -> Result<String> {
    tokio::task::spawn_blocking(move || {
        argon2::hash_encoded(
            plaintext_password.as_bytes(),
            nanoid::nanoid!(24).as_bytes(),
            &ARGON_CONFIG,
        )
    })
    .await
    .to_internal_error()?
    .to_internal_error()
}

pub async fn assert_safe(password: &str) -> Result<()> {
    // Make sure the password is long enough.
    if password.len() < 8 {
        return Err(create_error!(ShortPassword));
    }

    let config = config().await;

    if !config.api.security.easypwned.is_empty() {
        let mut hasher = sha1::Sha1::new();
        hasher.update(password);
        let pwd_hash = hasher.finalize();

        // easypwned expects the plain lowercase SHA-1 hex. The `#02` that used to be in this
        // specifier had no effect on `GenericArray`'s `LowerHex`, which is what clippy's
        // `unused_format_specs` says; the test below pins the string this produces.
        let result = match CLIENT
            .get(format!(
                "{}/hash/{pwd_hash:x}",
                config.api.security.easypwned
            ))
            .send()
            .await
        {
            Ok(response) => match response.json::<EasyPwnedResult>().await {
                Ok(result) => Ok(result.secure),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        };

        if let Err(e) = &result {
            sonm_config::capture_error(e);
        } else if result.is_ok_and(|b| b) {
            return Ok(());
        }
    };

    if TOP_100K_COMPROMISED.contains(password) {
        return Err(create_error!(CompromisedPassword));
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use sha1::{Digest, Sha1};

    /// The URL `assert_safe` builds is a contract with easypwned: 40 lowercase hex
    /// characters, no `0x` prefix, no padding.
    #[test]
    fn sha1_is_formatted_the_way_easypwned_wants() {
        let mut hasher = Sha1::new();
        hasher.update("password");

        assert_eq!(
            format!("{:x}", hasher.finalize()),
            "5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8"
        );
    }
}
