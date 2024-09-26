use anyhow::Result;
use itertools::Itertools;
use octocrab::params::{pulls::Sort, Direction, State};

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: also include lemmy-ui repo and merge results
    let page = octocrab::instance()
        .pulls("LemmyNet", "lemmy")
        .list()
        .state(State::Closed)
        .head("main")
        .sort(Sort::Updated)
        .direction(Direction::Descending)
        .per_page(100)
        .send()
        .await?;

    // TODO: use lemmy api to find date of last dev update (regex), 
    //       then consider all PRs since that time

    page.items
        .iter()
        .into_iter()
        .filter(|pr| pr.merged_at.is_some())
        // Ignore PRs with label `internal`
        // TODO: apply this to refactoring changes and similar
        .filter(|pr| pr.labels.clone().unwrap().iter().all(|l| l.name != "internal"))
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
