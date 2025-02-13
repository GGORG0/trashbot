use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct VoiceTime {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub user_id: u64,
    pub guild_id: u64,
    pub time: u64,
}
