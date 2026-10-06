use sonm_result::Result;

use crate::AccountInvite;

mod mongodb;

#[async_trait]
pub trait AbstractAccountInvites: Sync + Send {
    /// Find invite by id
    async fn fetch_account_invite(&self, id: &str) -> Result<AccountInvite>;

    /// Save invite
    async fn save_account_invite(&self, invite: &AccountInvite) -> Result<()>;
}
