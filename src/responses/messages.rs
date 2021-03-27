pub use serde::Deserialize;

use serde_json::Value;
use crate::responses::BasicThing;
use crate::responses::listing::ListingData;

pub type MessageListingData = BasicThing<ListingData<MessageData>>;

#[derive(Deserialize, Debug)]
pub struct MessageData {
    pub author: Option<String>,
    pub body: String,
    pub body_html: String,
    pub context: String,
    pub first_message_name: Option<String>,
    pub likes: Option<bool>,
    pub name: String,
    pub link_title: Option<String>,
    pub parent_id: Option<String>,
    pub replies: Value,
    pub subject: String,
    pub subreddit: Option<String>,
    pub was_comment: bool,
    pub created: f64,
    pub created_utc: f64
}
