//! Disable an account.
//! POST /account/disable
use rocket::State;
use rocket_empty::EmptyResponse;
use sonm_database::{Account, Database, ValidatedTicket};
use sonm_result::Result;

/// # Disable Account
///
/// Disable an account.
#[openapi(tag = "Account")]
#[post("/disable")]
pub async fn disable_account(
    db: &State<Database>,
    mut account: Account,
    _ticket: ValidatedTicket,
) -> Result<EmptyResponse> {
    account.disable(db).await.map(|_| EmptyResponse)
}

#[cfg(test)]
mod tests {
    use crate::util::test::PubSubTestHelper;
    use crate::{rocket, util::test::TestHarness};
    use rocket::http::{Header, Status};
    use sonm_database::{MFATicket, events::client::EventV1};
    use sonm_result::ErrorType;

    #[rocket::async_test]
    async fn success() {
        let harness = TestHarness::new().await;
        let (account, session, _) = harness.new_user().await;
        let mut pubsub = PubSubTestHelper::new(&format!("{}!", &account.id)).await;

        let ticket = MFATicket::new(account.id.to_string(), true);
        ticket.save(&harness.db).await.unwrap();

        let res = harness
            .client
            .post("/auth/account/disable")
            .header(Header::new("X-Session-Token", session.token.clone()))
            .header(Header::new("X-MFA-Ticket", ticket.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
        drop(res);
        assert!(
            harness
                .db
                .fetch_account(&account.id)
                .await
                .unwrap()
                .disabled
        );

        assert!(matches!(
            harness
                .db
                .fetch_session(&session.id)
                .await
                .unwrap_err()
                .error_type,
            ErrorType::UnknownUser
        ));

        pubsub
            .wait_for_event(|e| {
                if let EventV1::DeleteAllSessions { user_id, .. } = e {
                    user_id == &account.id
                } else {
                    false
                }
            })
            .await;
    }
}
