use anyhow::Result;
use generate_dev_update::last_dev_update;
use generate_dev_update::list_prs;
use itertools::Itertools;
use octocrab::models::pulls::PullRequest;
use tokio::try_join;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Generating change list for new dev update");
    let mut pull_requests: Vec<PullRequest> = vec![];

    let (mut lemmy_prs, mut lemmy_ui_prs, last_dev_update) =
        try_join!(list_prs("lemmy"), list_prs("lemmy-ui"), last_dev_update())?;
    pull_requests.append(&mut lemmy_prs);
    pull_requests.append(&mut lemmy_ui_prs);
    println!("Last dev update was at {}", last_dev_update.published);
    println!("\n{}", "=".repeat(100));

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
            println!("\n## {}\n", pr.0);
            for (_, pr) in pr.1 {
                println!(
                    "- [{}]({})",
                    pr.title.clone().unwrap().trim(),
                    pr.html_url.unwrap().as_str()
                );
            }
        });

    println!("\n");

    Ok(())
}
