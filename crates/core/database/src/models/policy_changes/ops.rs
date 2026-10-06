use sonm_result::Result;

use crate::PolicyChange;

mod mongodb;

#[async_trait]
pub trait AbstractPolicyChange: Sync + Send {
    /// Fetch all policy changes
    async fn fetch_policy_changes(&self) -> Result<Vec<PolicyChange>>;

    /// Acknowledge policy changes
    async fn acknowledge_policy_changes(&self, user_id: &str) -> Result<()>;
}
