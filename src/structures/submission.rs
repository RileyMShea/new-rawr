use serde_json;


use crate::responses::{listing, FlairSelectorResponse, FlairChoice};
use crate::client::RedditClient;
use crate::traits::{Votable, Editable, Created, Content, Approvable, Commentable, Stickable, Lockable, Reportable, Distinguishable, Flairable, Visible};
use crate::errors::APIError;
use crate::structures::user::User;
use crate::structures::subreddit::Subreddit;
use crate::responses::comment::{CommentData, NewComment};
use crate::structures::comment_list::{CommentList, CommentStream};
use crate::structures::listing::Listing;
use crate::structures::comment::Comment;
use crate::responses::listing::CommentResponse;

/// Structure representing a link post or self post (a submission) on Reddit.
pub struct Submission<'a> {
    data: listing::SubmissionData,
    client: &'a RedditClient,
}

impl<'a> PartialEq for Submission<'a> {
    fn eq(&self, other: &Submission) -> bool {
        self.name() == other.name()
    }
}


impl<'a> Votable for Submission<'a> {
    fn score(&self) -> i64 {
        self.data.score
    }

    fn likes(&self) -> Option<bool> {
        self.data.likes
    }

    fn upvote(&self) -> Result<(), APIError> {
        self.vote(1)
    }

    fn downvote(&self) -> Result<(), APIError> {
        self.vote(-1)
    }

    fn cancel_vote(&self) -> Result<(), APIError> {
        self.vote(0)
    }
}

impl<'a> Created for Submission<'a> {
    fn created(&self) -> i64 {
        self.data.created as i64
    }

    fn created_utc(&self) -> i64 {
        self.data.created_utc as i64
    }
}

impl<'a> Editable for Submission<'a> {
    fn edited(&self) -> bool {
        self.data.edited.as_bool().unwrap()
    }

    fn edited_time(&self) -> Option<i64> {
        self.data.edited.as_i64()
    }

    fn edit(&mut self, text: &str) -> Result<(), APIError> {
        let body = format!("api_type=json&text={}&thing_id={}",
                           self.client.url_escape(text.to_owned()),
                           self.data.name);
        let res = self.client.post_success("/api/editusertext", &body, false);
        if let Ok(()) = res {
            // TODO: should we update selftext_html?
            self.data.selftext = text.to_owned();
        }
        res
    }

    fn body(&self) -> Option<String> {
        let self_text = self.data.selftext.to_owned();
        if self_text.is_empty() {
            None
        } else {
            Some(self_text)
        }
    }

    fn body_html(&self) -> Option<String> {
        self.data.selftext_html.to_owned()
    }
}

impl<'a> Content for Submission<'a> {
    fn author(&self) -> User {
        User::new(self.client, &self.data.author)
    }

    fn author_flair_text(&self) -> Option<String> {
        self.data.author_flair_text.to_owned()
    }

    fn author_flair_css(&self) -> Option<String> {
        self.data.author_flair_css_class.to_owned()
    }

    fn subreddit(&self) -> Subreddit {
        Subreddit::create_new(self.client, &self.data.subreddit)
    }

    fn delete(self) -> Result<(), APIError> {
        let body = format!("id={}", self.data.name);
        self.client.post_success("/api/del", &body, false)
    }
    fn name(&self) -> &str {
        &self.data.name
    }
}

impl<'a> Approvable for Submission<'a> {
    fn approve(&self) -> Result<(), APIError> {
        let body = format!("id={}", self.data.name);
        self.client.post_success("/api/approve", &body, false)
    }

    fn remove(&self, spam: bool) -> Result<(), APIError> {
        let body = format!("id={}&spam={}", self.data.name, spam);
        self.client.post_success("/api/remove", &body, false)
    }

    fn ignore_reports(&self) -> Result<(), APIError> {
        let body = format!("id={}", self.data.name);
        self.client.post_success("/api/ignore_reports", &body, false)
    }

    fn unignore_reports(&self) -> Result<(), APIError> {
        let body = format!("id={}", self.data.name);
        self.client.post_success("/api/unignore_reports", &body, false)
    }
}

impl<'a> Commentable<'a> for Submission<'a> {
    fn reply_count(&self) -> u64 {
        self.data.num_comments
    }

    fn reply(&self, text: &str) -> Result<Comment, APIError> {
        let body = format!("api_type=json&text={}&thing_id={}",
                           self.client.url_escape(text.to_owned()),
                           self.name());
        //
        let result = self.client.post_json("/api/comment", &body, false).unwrap();
        let result: NewComment = serde_json::from_str(&*result).unwrap();

        Ok(Comment::new(self.client, result.json.data.things.into_iter().next().unwrap().data))
    }

    fn replies(self) -> Result<CommentList<'a>, APIError> {
        // TODO: sort type
        let url = format!("/comments/{}", self.data.id);
        let result = self.client.get_json(&url, false).unwrap();
        let result: listing::CommentResponse = serde_json::from_str(&*result).unwrap();

        Ok(CommentList::new(self.client,
                            self.data.name.to_owned(),
                            self.data.name.to_owned(),
                            result.1.data.children))
    }
}

impl<'a> Submission<'a> {
    /// Internal method. Get submissions from a listing instead (see `Subreddit.hot()` etc.)
    pub fn new(client: &RedditClient, data: listing::SubmissionData) -> Submission {
        Submission {
            client: client,
            data: data,
        }
    }

    /// Returns a `CommentStream` that fetches the latest comments in an infinite loop and returns
    /// it from the iterator. Comments will be ordered from oldest to newest, with up to 5 comments
    /// that exist being yielded at a time. This will poll the API every 5 seconds for updates.
    /// # Examples
    /// ```rust,no_run
    /// use new_rawr::prelude::*;
    /// let client = RedditClient::new("new_rawr", AnonymousAuthenticator::new());
    /// let sub = client.subreddit("all");
    /// let mut listing = sub.hot(ListingOptions::default()).expect("Could not fetch listing!");
    /// let post = listing.nth(0).unwrap();
    /// for comment in post.reply_stream() {
    ///     println!("New comment received!");
    /// }
    pub fn reply_stream(self) -> CommentStream<'a> {
        CommentStream::new(self.client, self.data.name, self.data.id)
    }

    /// The title of the post (as an &str). All link and self posts have a title, and any post
    /// flairs are not included in this.
    pub fn title(&self) -> &str {
        &self.data.title
    }

    /// This is `true` if the post is a self post, and `false` if it is a link post.
    pub fn is_self_post(&self) -> bool {
        self.data.is_self
    }

    /// Gets the URL linked to by this link post (or `None`, if this is a self post)
    pub fn link_url(&self) -> Option<String> {
        self.data.url.to_owned()
    }

    /// Returns `true` if the post is marked NSFW (over 18).
    pub fn nsfw(&self) -> bool {
        self.data.over_18
    }

    /// Sets the post as NSFW (over 18) if you have the correct privileges (owner of the post or
    /// moderator) **and** the subreddit allows NSFW posts.
    pub fn mark_nsfw(&mut self) -> Result<(), APIError> {
        let body = format!("id={}", self.data.name);
        let res = self.client.post_success("/api/marknsfw", &body, false);

        if let Ok(_) = res {
            self.data.over_18 = true;
        }

        res
    }

    /// Sets the post as **not** NSFW (over 18).
    pub fn unmark_nsfw(&mut self) -> Result<(), APIError> {
        let body = format!("id={}", self.data.name);
        let res = self.client.post_success("/api/unmarknsfw", &body, false);

        if let Ok(_) = res {
            self.data.over_18 = false;
        }

        res
    }

    fn vote(&self, dir: i8) -> Result<(), APIError> {
        let body = format!("dir={}&id={}", dir, self.data.name);
        self.client.post_success("/api/vote", &body, false)
    }
}

impl<'a> Stickable for Submission<'a> {
    /// This is `true` if the post is stickied (an announcement post).
    fn stickied(&self) -> bool {
        self.data.stickied
    }

    fn stick(&mut self) -> Result<(), APIError> {
        let body = format!("api_type=json&id={}&state=true", self.data.name);
        let res = self.client.post_success("/api/set_subreddit_sticky", &body, false);

        if let Ok(_) = res {
            self.data.stickied = true;
        }

        res
    }

    fn unstick(&mut self) -> Result<(), APIError> {
        let body = format!("api_type=json&id={}&state=false", self.data.name);
        let res = self.client.post_success("/api/set_subreddit_sticky", &body, false);

        if let Ok(_) = res {
            self.data.stickied = false;
        }

        res
    }
}

impl<'a> Lockable for Submission<'a> {
    fn locked(&self) -> bool {
        self.data.locked
    }

    fn lock(&mut self) -> Result<(), APIError> {
        let body = format!("id={}", self.data.name);
        let res = self.client.post_success("/api/lock", &body, false);

        if let Ok(_) = res {
            self.data.locked = true;
        }

        res
    }

    fn unlock(&mut self) -> Result<(), APIError> {
        let body = format!("id={}", self.data.name);
        let res = self.client.post_success("/api/unlock", &body, false);

        if let Ok(_) = res {
            self.data.locked = false;
        }

        res
    }
}

impl<'a> Reportable for Submission<'a> {
    fn report(&self, reason: &str) -> Result<(), APIError> {
        let body = format!("api_type=json&thing_id={}&reason={}",
                           self.data.name,
                           self.client.url_escape(reason.to_owned()));
        self.client.post_success("/api/report", &body, false)
    }

    fn report_count(&self) -> Option<u64> {
        self.data.num_reports.to_owned()
    }
}

impl<'a> Distinguishable for Submission<'a> {
    fn distinguished(&self) -> Option<String> {
        self.data.distinguished.to_owned()
    }

    fn distinguish(&mut self) -> Result<(), APIError> {
        let body = format!("api_type=json&how=yes&id={}", self.data.name);
        let res = self.client.post_success("/api/distinguish", &body, false);
        if let Ok(()) = res {
            self.data.distinguished = Some(String::from("moderator"));
        }
        res
    }

    fn undistinguish(&mut self) -> Result<(), APIError> {
        let body = format!("api_type=json&how=no&id={}", self.data.name);
        let res = self.client.post_success("/api/distinguish", &body, false);
        if let Ok(()) = res {
            self.data.distinguished = None;
        }
        res
    }
}

impl<'a> Flairable for Submission<'a> {
    fn get_flair_text(&self) -> Option<String> {
        self.data.link_flair_text.to_owned()
    }

    fn get_flair_css(&self) -> Option<String> {
        self.data.link_flair_css_class.to_owned()
    }

    fn flair_options(&self) -> Result<FlairList, APIError> {
        let body = format!("link={}", self.data.name);
        let url = format!("/r/{}/api/flairselector", self.data.subreddit);
        let result = self.client
            .post_json(&url, &body, false).unwrap();
        let result: FlairSelectorResponse = serde_json::from_str(&*result).unwrap();
        Ok(FlairList::new(result.choices))
    }

    fn flair(&self, template: &str) -> Result<(), APIError> {
        let body = format!("api_type=json&link={}&flair_template_id={}",
                           self.data.name,
                           template);
        let url = format!("/r/{}/api/selectflair", self.data.subreddit);
        self.client.post_success(&url, &body, false)
    }
}

impl<'a> Visible for Submission<'a> {
    fn hidden(&self) -> bool {
        self.data.hidden
    }

    fn hide(&mut self) -> Result<(), APIError> {
        let body = format!("id={}", self.data.name);
        let res = self.client.post_success("/api/hide", &body, false);

        if let Ok(_) = res {
            self.data.hidden = true;
        }

        res
    }

    fn show(&mut self) -> Result<(), APIError> {
        let body = format!("id={}", self.data.name);
        let res = self.client.post_success("/api/unhide", &body, false);

        if let Ok(_) = res {
            self.data.hidden = false;
        }

        res
    }
}

/// A list of flairs that can be assigned to a post. To access the complete list, use
/// `FlairList.flairs`, which is a list of `FlairChoice` objects.
pub struct FlairList {
    /// The list of flairs available.
    pub flairs: Vec<FlairChoice>,
}

impl FlairList {
    /// Creates a `FlairList` from a vector of `FlairChoice` objects.
    pub fn new(choices: Vec<FlairChoice>) -> FlairList {
        FlairList { flairs: choices }
    }

    /// Finds the flair with the specified text, consuming the `FlairList`.
    /// # Examples
    /// ```rust,no_run
    /// use new_rawr::client::RedditClient;
    /// use new_rawr::auth::PasswordAuthenticator;
    /// use new_rawr::options::ListingOptions;
    /// use new_rawr::traits::Flairable;
    /// let client = RedditClient::new("new_rawr", PasswordAuthenticator::new("a", "b", "c", "d"));
    /// let sub = client.subreddit("learnprogramming");
    /// let post = sub.hot(ListingOptions::default()).unwrap().next().unwrap();
    /// // NOTE: this would 403 unless you are a moderator or the creator of the post.
    /// let tutorial_flair = post.flair_options().unwrap().find_text("tutorial").unwrap();
    /// post.flair(&tutorial_flair);
    /// ```
    pub fn find_text(self, text: &str) -> Option<String> {
        for flair in self.flairs {
            if flair.flair_text == text {
                return Some(flair.flair_template_id);
            }
        }

        None
    }
}

/// A lazy object representing a submission. Used by the `Client.get_by_id()` method until the
/// data is specified by the user (we don't know if they want the `Submission` or `CommentList`
/// yet). The `LazySubmission` object is consumed when performing either of these actions.
pub struct LazySubmission<'a> {
    id: String,
    client: &'a RedditClient,
}

impl<'a> LazySubmission<'a> {
    /// Internal method. Use `Client.get_by_id()` instead.
    pub fn new(client: &'a RedditClient, id: &str) -> LazySubmission<'a> {
        LazySubmission {
            client: client,
            id: id.to_owned(),
        }
    }

    /// Fetches the `Submission` with this ID, in order to access post title, body, link and
    /// creation time.
    pub fn get(self) -> Result<Submission<'a>, APIError> {
        let url = format!("/by_id/{}?raw_json=1", self.id);
        let string = self.client
            .get_json(&url, false).unwrap();
        let mut string: listing::Listing = serde_json::from_str(&*string).unwrap();
        let mut string = Listing::new(self.client, url, string.data);
        Ok(string.next().unwrap())
    }

    /// Fetches a `CommentList` with replies to this submission.
    pub fn replies(self) -> Result<CommentList<'a>, APIError> {
        let url = format!("/comments/{}?raw_json=1", self.id.split('_').nth(1).unwrap());
        let string = self.client
            .get_json(&url, false).unwrap();
        let string :listing::CommentResponse =serde_json::from_str(&*string).unwrap();
        Ok(CommentList::new(self.client,
                            self.id.to_owned(),
                            self.id.to_owned(),
                            string.1.data.children))
    }
}
