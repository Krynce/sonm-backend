//! Fetch your account
//! GET /account
use rocket::serde::json::Json;
use sonm_database::Account;
use sonm_models::v0;
use sonm_result::Result;

/// # Fetch Account
///
/// Fetch account information from the current session.
#[openapi(tag = "Account")]
#[get("/")]
pub async fn fetch_account(account: Account) -> Result<Json<v0::AccountInfo>> {
    Ok(Json(account.into()))
}

#[cfg(test)]
mod tests {
    use crate::{rocket, util::test::TestHarness};
    use rocket::http::{Header, Status};
    use sonm_models::v0;

    #[rocket::async_test]
    async fn success() {
        let harness = TestHarness::new().await;
        let (account, session, _) = harness.new_user().await;

        let res = harness
            .client
            .get("/auth/account")
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert_eq!(
            &res.into_json::<v0::AccountInfo>().await.unwrap().id,
            &account.id
        );
    }
}
