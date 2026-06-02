use anyhow::Result;
use clap::Parser;
use futures_util::future::try_join_all;
use generate_dev_update::{list_prs, string_to_utc};
use itertools::Itertools;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
  /// The start date for the pull request range. (IE 2024-10-01)
  start_date: String,
  /// The end date for the pull request range. (IE 2024-11-01)
  end_date: String,
}

#[tokio::main]
async fn main() -> Result<()> {
  rustls::crypto::ring::default_provider()
    .install_default()
    .unwrap();

  let cli = Cli::parse();

  let start_date = string_to_utc(&cli.start_date)?;
  let end_date = string_to_utc(&cli.end_date)?;

  println!(
    "# Dev Update from {} to {}",
    start_date.date_naive(),
    end_date.date_naive()
  );

  try_join_all([
    list_prs("lemmy", &start_date, &end_date),
    list_prs("lemmy-ui", &start_date, &end_date),
    list_prs("joinlemmy-site", &start_date, &end_date),
    list_prs("jerboa", &start_date, &end_date),
    list_prs("lemmy-js-client", &start_date, &end_date),
    list_prs("lemmy-client-rs", &start_date, &end_date),
  ])
  .await?
  .into_iter()
  .for_each(|pr| {
    pr.into_iter()
      .map(|pr| (pr.head.repo.clone().unwrap().name, pr))
      .sorted_by(|a, b| Ord::cmp(&b.0, &a.0))
      // Group by repo name
      .chunk_by(|(repo, _)| repo.clone())
      .into_iter()
      .map(|chunk| (chunk.0, chunk.1.collect::<Vec<_>>()))
      // Print as markdown
      .for_each(|pr| {
        println!("\n## {}\n", pr.0);
        for (_, pr) in pr.1 {
          println!(
            "- [{}]({}) by @{}",
            pr.title.clone().unwrap().trim(),
            pr.html_url.clone().unwrap().as_str(),
            pr.user.clone().unwrap().login
          );
        }
      });
  });

  println!("\n");

  Ok(())
}
