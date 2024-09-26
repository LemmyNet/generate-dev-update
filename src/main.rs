use anyhow::Result;
use itertools::Itertools;
use lemmy_api_common::{lemmy_db_schema::source::post::Post, post::GetPostsResponse};
use octocrab::{
    models::pulls::PullRequest,
    params::{pulls::Sort, Direction, State},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Generating change list for new dev update");
    let mut pull_requests = list_prs("lemmy").await?;
    pull_requests.append(&mut list_prs("lemmy-ui").await?);

    let last_dev_update = last_dev_update().await?;
    println!("Last dev update was at {}", last_dev_update.published);

    pull_requests
        .into_iter()
        .filter(|pr| pr.merged_at.unwrap_or_default() > last_dev_update.published)
        // Ignore PRs with label `internal`
        // TODO: apply this to refactoring changes and similar
        .filter(|pr| {
            pr.labels
                .clone()
                .unwrap()
                .iter()
                .all(|l| l.name != "internal")
        })
        .map(|pr| (pr.user.clone().unwrap().login, pr))
        .sorted_by(|a, b| Ord::cmp(&b.0, &a.0))
        // Ignore dependency updates
        .filter(|(author, _)| author != "renovate[bot]")
        // Group by author name
        .chunk_by(|(author, _)| author.clone())
        .into_iter()
        .map(|chunk| (chunk.0, chunk.1.collect::<Vec<_>>()))
        // Show authors with less PRs first
        .sorted_by(|a, b| Ord::cmp(&a.1.len(), &b.1.len()))
        // Print as markdown
        .for_each(|pr| {
            println!("\n\n## {}\n\n", pr.0);
            for (_, pr) in pr.1 {
                println!("[{}]({})", pr.title.clone().unwrap().trim(), pr.url,);
            }
        });

    Ok(())
}

/// Get list of pull requests from given repo under LemmyNet
async fn list_prs(repo: &str) -> Result<Vec<PullRequest>> {
    Ok(octocrab::instance()
        .pulls("LemmyNet", repo)
        .list()
        .state(State::Closed)
        .head("main")
        .sort(Sort::Updated)
        .direction(Direction::Descending)
        .per_page(100)
        .send()
        .await?
        .items)
}

// Use lemmy api to find last dev update post
async fn last_dev_update() -> Result<Post> {
    let client = reqwest::Client::builder()
        .user_agent("generate-dev-update")
        .build()?;
    let res = client.get("https://lemmy.ml/api/v3/post/list?limit=20&sort=New&type_=All&community_name=announcements").send().await?
    .json::<GetPostsResponse>().await?;
    Ok(res
        .posts
        .into_iter()
        .map(|p| p.post)
        .filter(|p| p.name.contains("Lemmy Development Update"))
        .next()
        .unwrap())
}