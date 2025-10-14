use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name="jozin", version, about="Local photo organizer")]
struct Args {
  #[command(subcommand)]
  cmd: Cmd
}
#[derive(Subcommand)]
enum Cmd {
  Scan { path: String, #[arg(long)] dry_run: bool },
  Verify { path: String },
  Migrate { path: String, #[arg(long)] to: Option<String> },
}

fn main() -> Result<()> {
  let args = Args::parse();
  match args.cmd {
    Cmd::Scan { path, dry_run } => {
      if dry_run { println!("DRY RUN: {}", path); return Ok(()); }
      let out = jozin_core::api::scan_path(&path)?;
      println!("{}", serde_json::to_string_pretty(&out)?);
    }
    Cmd::Verify { path } => { jozin_core::api::verify_path(&path)?; println!("OK"); }
    Cmd::Migrate { path, to } => { println!("Migrate {} -> {:?}", path, to); }
  }
  Ok(())
}
