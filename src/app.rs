use std::time::{Duration, Instant};
use std::fs;
use rand::seq::SliceRandom;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

use crate::config::Config;
use crate::error::{AppError, Result};
use crate::ui::UI;

// Embed the default team.txt file at compile time
const DEFAULT_TEAM_CONTENT: &str = include_str!("../team.txt");

/// Main application state
pub struct App {
    config: Config,
    names: Vec<String>,
    per_person_timers: Vec<Duration>,
    current_person_index: usize,
    timer_start: Instant,
    last_ppt_update: Instant,
    should_quit: bool,
    is_dark_background: bool,
}

impl App {
    /// Create a new application instance
    pub async fn new(config: Config) -> Result<Self> {
        let names = Self::load_names(&config.names_file)?;

        if names.is_empty() {
            return Err(AppError::NoNamesFound.into());
        }

        let per_person_timers = vec![Duration::ZERO; names.len()];

        // Detect terminal background (default to dark if detection fails)
        let is_dark_background = Self::detect_dark_background().unwrap_or(true);

        Ok(Self {
            config,
            names,
            per_person_timers,
            current_person_index: 0,
            timer_start: Instant::now(),
            last_ppt_update: Instant::now(),
            should_quit: false,
            is_dark_background
        })
    }

    /// Attempt to detect if terminal has a dark background.
    /// Returns None if detection fails, Some(true) for dark, Some(false) for light
    fn detect_dark_background() -> Option<bool> {
        use std::io::Write;
        use std::time::Duration as StdDuration;

        // Try to query terminal background color using OSC 11
        // Not all terminals support this, so we'll use a timeout
        let mut stdout = io::stdout();

        // Send OSC 11 query (request background color)
        if write!(stdout, "\x1b]11;?\x1b\\").is_err() {
            return None;
        }
        if stdout.flush().is_err() {
            return None;
        }

        // Try to read response with timeout
        // This is a simple heuristic; if we can't detect, we'll default to dark
        if let Ok(true) = event::poll(StdDuration::from_millis(100)) {
            if let Ok(Event::Key(_)) = event::read() {
                // If we got any response, try to parse it
                // This is a simplified check - in practice, OSC responses are complex
                // For now, we'll use an environment variable as a more reliable fallback
            }
        }

        // Fallback: Check common environment variables
        if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
            // Some terminal emulators set helpful env vars
            if term_program.contains("light") {
                return Some(false);
            }
        }

        // Check COLORFGBG (set by some terminals: "foreground;background")
        if let Ok(colorfgbg) = std::env::var("COLORFGBG") {
            if let Some(bg) = colorfgbg.split(';').last() {
                if let Ok(bg_num) = bg.parse::<u8>() {
                    // In COLORFGBG, lower numbers (0-7) typically mean dark colors
                    // Higher numbers (8-15) typically mean light colors
                    return Some(bg_num < 8);
                }
            }
        }

        // Default assumption: dark background (most common for terminals)
        Some(true)
    }

    /// Load names from a file, falling back to embedded default if file not found
    fn load_names(filename: &str) -> Result<Vec<String>> {
        // Try to read from file first
        let content = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                // If the file doesn't exist, and we're using the default filename,
                // fall back to the embedded content
                if filename == "team.txt" && e.kind() == io::ErrorKind::NotFound {
                    DEFAULT_TEAM_CONTENT.to_string()
                } else {
                    // For other errors or custom filenames, propagate the error
                    return Err(AppError::NamesFileError(e).into());
                }
            }
        };

        let names: Vec<String> = content
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        Ok(names)
    }

    /// Shuffle the names randomly
    fn shuffle_names(&mut self) {
        let mut rng = rand::rng();
        self.names.shuffle(&mut rng);
        self.reset_per_person_timers();
    }

    /// Reset per-person timers
    fn reset_per_person_timers(&mut self) {
        self.per_person_timers = vec![Duration::ZERO; self.names.len()];
        self.current_person_index = 0;
    }

    /// Reset the main timer
    fn reset_timer(&mut self) {
        self.timer_start = Instant::now();
        self.last_ppt_update = Instant::now();
    }

    /// Update per-person timers
    fn update_per_person_timers(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_ppt_update);

        // Update the current person's timer
        if self.current_person_index < self.per_person_timers.len() {
            self.per_person_timers[self.current_person_index] += elapsed;
        }

        self.last_ppt_update = now;
    }

    /// Get remaining meeting time
    pub fn remaining_time(&self) -> Duration {
        let elapsed = self.timer_start.elapsed();
        if elapsed >= self.config.duration {
            Duration::ZERO
        } else {
            self.config.duration - elapsed
        }
    }

    /// Handle keyboard input
    fn handle_input(&mut self, key: event::KeyEvent) -> Result<()> {
        match (key.code, key.modifiers) {
            // Ctrl+R -- Reset timer and per-person timers
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                self.reset_per_person_timers();
                self.reset_timer();
            }

            // Ctrl+N -- Reshuffle names
            (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                self.shuffle_names();
                self.reset_timer();
            }

            // Ctrl+C or 'q' -- Quit
            (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.should_quit = true;
            }

            // Tab or Down Arrow -- Next person
            (KeyCode::Tab, KeyModifiers::NONE) | (KeyCode::Down, KeyModifiers::NONE) => {
                if self.current_person_index < self.names.len() - 1 {
                    self.current_person_index += 1;
                }
            }

            // Shift+Tab or Up Arrow -- Previous person
            (KeyCode::BackTab, KeyModifiers::NONE) | (KeyCode::Up, KeyModifiers::NONE) => {
                if self.current_person_index > 0 {
                    self.current_person_index -= 1;
                }
            }

            _ => {}
        }

        Ok(())
    }

    /// Main application loop
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal -- ratatui's way of controlling terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Shuffle names initially
        self.shuffle_names();

        // Main event loop
        let res = self.run_app(&mut terminal).await;

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        res
    }

    /// Internal run loop that handles events and rendering
    async fn run_app(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            // Update timers
            self.update_per_person_timers();

            // Render UI
            let ui = UI::new(self);
            terminal.draw(|f| ui.render(f))?;

            // Handle input with timeout to allow for regular updates
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_input(key)?;
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    // Getter methods for UI access
    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn per_person_timers(&self) -> &[Duration] {
        &self.per_person_timers
    }

    pub fn current_person_index(&self) -> usize {
        self.current_person_index
    }

    pub fn is_dark_background(&self) -> bool {
        self.is_dark_background
    }
}