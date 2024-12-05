use anyhow::Result;
use lemmy_api_common::{lemmy_db_schema::source::post::Post, post::GetPostsResponse};
use octocrab::{
    models::pulls::PullRequest,
    params::{pulls::Sort, Direction, State},
};

/// Get list of pull requests from given repo under LemmyNet
pub async fn list_prs(repo: &str) -> Result<Vec<PullRequest>> {
    Ok(octocrab::instance()
        .pulls("LemmyNet", repo)
        .list()
        .state(State::Closed)
        .base("main")
        .sort(Sort::Updated)
        .direction(Direction::Descending)
        .per_page(100)
        .send()
        .await?
        .items)
}

// Use lemmy api to find last dev update post
pub async fn last_dev_update() -> Result<Post> {
    let client = reqwest::Client::builder()
        .user_agent("generate-dev-update")
        .build()?;
    let url = "https://lemmy.ml/api/v3/post/list?limit=20&sort=New&type=Local&community_name=announcements";
    let res = client
        .get(url)
        .send()
        .await?
        .json::<GetPostsResponse>()
        .await?;
    Ok(res
        .posts
        .into_iter()
        .map(|p| p.post)
        .find(|p| p.name.contains("Lemmy Development Update"))
        .unwrap())
}
