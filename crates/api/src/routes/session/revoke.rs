//! Revoke an active session
//! DELETE /session/:id
use rocket::State;
use rocket_empty::EmptyResponse;
use sonm_database::{Database, Session, ValidatedTicket};
use sonm_result::{Result, create_error};

/// # Revoke Session
///
/// Delete a specific active session.
#[openapi(tag = "Session")]
#[delete("/<id>")]
pub async fn revoke(
    db: &State<Database>,
    _ticket: ValidatedTicket,
    user: Session,
    id: String,
) -> Result<EmptyResponse> {
    let session = db.fetch_session(&id).await?;

    if session.user_id != user.user_id {
        return Err(create_error!(InvalidToken));
    }

    session.delete(db).await.map(|_| EmptyResponse)
}

#[cfg(test)]
mod tests {
    use crate::{rocket, util::test::TestHarness};
    use rocket::http::{Header, Status};
    use sonm_database::{MFATicket, Totp};
    use sonm_result::ErrorType;

    #[rocket::async_test]
    async fn success() {
        let harness = TestHarness::new().await;
        let (account, session, _) = harness.new_user().await;

        let ticket = MFATicket::new(account.id.to_string(), true);
        ticket.save(&harness.db).await.unwrap();

        let res = harness
            .client
            .delete(format!("/auth/session/{}", session.id))
            .header(Header::new("X-Session-Token", session.token))
            .header(Header::new("X-MFA-Ticket", ticket.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
        assert!(matches!(
            harness
                .db
                .fetch_session(&session.id)
                .await
                .unwrap_err()
                .error_type,
            ErrorType::UnknownUser
        ));
    }

    #[rocket::async_test]
    async fn success_mfa() {
        let harness = TestHarness::new().await;
        let (mut account, session, _) = harness.new_user().await;

        let totp = Totp::Enabled {
            secret: "secret".to_string(),
        };

        account.mfa.totp_token = totp.clone();
        account.save(&harness.db).await.unwrap();

        let ticket = MFATicket::new(account.id.to_string(), true);
        ticket.save(&harness.db).await.unwrap();

        let res = harness
            .client
            .delete(format!("/auth/session/{}", session.id))
            .header(Header::new("X-Session-Token", session.token))
            .header(Header::new("X-MFA-Ticket", ticket.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
        assert!(matches!(
            harness
                .db
                .fetch_session(&session.id)
                .await
                .unwrap_err()
                .error_type,
            ErrorType::UnknownUser
        ));
    }

    #[rocket::async_test]
    async fn fail_mfa() {
        let harness = TestHarness::new().await;
        let (mut account, session, _) = harness.new_user().await;

        let totp = Totp::Enabled {
            secret: "secret".to_string(),
        };

        account.mfa.totp_token = totp.clone();
        account.save(&harness.db).await.unwrap();

        let res = harness
            .client
            .delete(format!("/auth/session/{}", session.id))
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Unauthorized);
    }
}
