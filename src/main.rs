use anyhow::Result;
use clap::{command, Parser};
use generate_dev_update::{list_prs, string_to_utc};
use itertools::Itertools;
use tokio::try_join;

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

  let (lemmy_prs, lemmy_ui_prs) = try_join!(
    list_prs("lemmy", &start_date, &end_date),
    list_prs("lemmy-ui", &start_date, &end_date)
  )?;

  let pull_requests = [lemmy_prs, lemmy_ui_prs].concat();

  pull_requests
    .into_iter()
    .map(|pr| (pr.user.clone().unwrap().login, pr))
    .sorted_by(|a, b| Ord::cmp(&b.0, &a.0))
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
