use anyhow::Result;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use octocrab::{
  models::pulls::PullRequest,
  params::{pulls::Sort, Direction, State},
};

/// Get list of pull requests from given repo under LemmyNet
pub async fn list_prs(
  repo: &str,
  start_date: &DateTime<Utc>,
  end_date: &DateTime<Utc>,
) -> Result<Vec<PullRequest>> {
  let mut results: Vec<PullRequest> = Vec::new();

  let mut current_date = Utc::now();
  let mut page = 1u32;

  // Keep fetching until before start date
  while current_date > *start_date {
    let mut fetch_results = octocrab::instance()
      .pulls("LemmyNet", repo)
      .list()
      .state(State::Closed)
      .base("main")
      .sort(Sort::Updated)
      .direction(Direction::Descending)
      .per_page(100)
      .page(page)
      .send()
      .await?
      .items;

    // Set the current date and increase the page.
    current_date = fetch_results
      .last()
      .map(|pr| pr.updated_at.unwrap_or_default())
      .unwrap_or_default();
    page += 1;

    results.append(&mut fetch_results);
  }

  let filtered_results: Vec<PullRequest> = results
    .into_iter()
    // Filter results to the current range
    .filter(|pr| {
      pr.merged_at.unwrap_or_default() > *start_date && pr.merged_at.unwrap_or_default() < *end_date
    })
    // Ignore PRs with label `internal`
    // TODO: apply this to refactoring changes and similar
    .filter(|pr| {
      pr.labels
        .clone()
        .unwrap()
        .iter()
        .all(|l| l.name != "internal")
    })
    // Ignore dependency updates
    .filter(|pr| {
      pr.user
        .clone()
        .map(|u| u.login != "renovate[bot]")
        .unwrap_or(false)
    })
    .collect();

  Ok(filtered_results)
}

pub fn string_to_utc(date_str: &str) -> Result<DateTime<Utc>> {
  Ok(
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?
      .and_time(NaiveTime::default())
      .and_utc(),
  )
}
