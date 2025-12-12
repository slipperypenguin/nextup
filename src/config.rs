use std::time::Duration;

// Configuration structure for the app
#[derive(Debug, Clone)]
pub struct Config {
    pub title: String,
    pub names_file: String,
    pub duration: Duration,
    pub hide_timer: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: "Team daily standup".to_string(),
            names_file: "team.txt".to_string(),
            duration: Duration::from_secs(15 * 60), // 15min
            hide_timer: false,
        }
    }
}
