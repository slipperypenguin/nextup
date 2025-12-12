use clap::Parser;
use std::time::Duration;

mod app;
mod config;
mod error;
mod ui;

use app::App;
use config::Config;
use error::Result;

#[derive(Parser)]
#[command(name = "nextup")]
#[command(version)]
#[command(about = "A simple tool that randomizes a list of names for daily standups.")]
struct Args {
    // Window title
    #[arg(long, default_value = "Team daily standup")]
    title: String,

    // Path to file with team member names
    #[arg(long, default_value = "team.txt")]
    names: String,

    // Meeting duration in minutes
    #[arg(long, default_value_t = 15)]
    duration: u64,

    // Hide timer
    #[arg(long, default_value_t = false)]
    hide_timer: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Convert CLI args to our config struct
    let config = Config {
        title: args.title,
        names_file: args.names,
        duration: Duration::from_secs(args.duration * 60), // convert minutes to seconds
        hide_timer: args.hide_timer,
    };

    // Initialize + Run the app
    let mut app = App::new(config).await?;
    app.run().await?;
    Ok(())
}
