use crate::{Account, Database, Session};
use rocket::{
    Request,
    http::Status,
    request::{FromRequest, Outcome},
};
use sonm_result::Error;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Account {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match request.guard::<Session>().await {
            Outcome::Success(session) => {
                if let Ok(account) = request
                    .rocket()
                    .state::<Database>()
                    .expect("`Database`")
                    .fetch_account(&session.user_id)
                    .await
                {
                    Outcome::Success(account)
                } else {
                    Outcome::Error((Status::InternalServerError, create_error!(InternalError)))
                }
            }
            Outcome::Forward(_) => unreachable!(),
            Outcome::Error(err) => Outcome::Error(err),
        }
    }
}
