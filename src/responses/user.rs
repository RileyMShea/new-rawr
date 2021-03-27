
/// API response for /user/username/about
pub type UserAbout = BasicThing<UserAboutData>;

pub use serde::Deserialize;
use crate::responses::BasicThing;

#[derive(Deserialize, Debug)]
pub struct UserAboutData {
    pub name: String,
    pub is_friend: bool,
    pub hide_from_robots: bool,
    pub id: String,
    pub created: f64,
    pub created_utc: f64,
    pub link_karma: i64,
    pub comment_karma: i64,
    pub is_gold: bool,
    pub is_mod: bool,
    pub has_verified_email: bool,
}
