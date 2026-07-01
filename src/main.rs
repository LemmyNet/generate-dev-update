use anyhow::Result;
use chrono::Month;
use clap::Parser;
use futures_util::future::try_join_all;
use generate_dev_update::{list_prs, string_to_utc};
use itertools::Itertools;
use std::str::FromStr;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
  date: String,
}

#[tokio::main]
async fn main() -> Result<()> {
  rustls::crypto::ring::default_provider()
    .install_default()
    .unwrap();

  let cli = Cli::parse();
  const PARSE_ERROR_MSG: &str = "Date must be formatted like '2026-5'";
  let (year, month) = cli.date.split_once('-').expect(PARSE_ERROR_MSG);
  let year = i32::from_str(year).expect(PARSE_ERROR_MSG);
  let month = Month::try_from(u8::from_str(month).expect(PARSE_ERROR_MSG))?;
  let month_num = month.number_from_month();
  let month_days = month.num_days(year).unwrap();
  let start_date = string_to_utc(&format!("{year}-{month_num}-01"))?;
  let end_date = string_to_utc(&format!("{year}-{month_num}-{month_days}"))?;

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
