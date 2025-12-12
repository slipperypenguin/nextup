use crate::app::App;
use ratatui::text::Span;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
};
use std::time::Duration;

/// UI renderer
pub struct UI<'a> {
    app: &'a App,
}

impl<'a> UI<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }

    /// Main render function
    pub fn render(&self, f: &mut Frame) {
        let config = self.app.config();

        // create the main layout
        let chunks = if config.hide_timer {
            // without timer: names and help
            Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Min(3),    // Names widget (flexible)
                        Constraint::Length(3), // Help widget (fixed)
                    ]
                    .as_ref(),
                )
                .split(f.area())
        } else {
            // with timer: names, timer, and help
            Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Min(3),    // Names widget (flexible)
                        Constraint::Length(5), // Timer widget (fixed)
                        Constraint::Length(3), // Help widget (fixed)
                    ]
                    .as_ref(),
                )
                .split(f.area())
        };

        // Render names widget
        self.render_names_widget(f, chunks[0]);

        // Render timer widget if not hidden
        if !config.hide_timer {
            self.render_timer_widget(f, chunks[1]);
            self.render_help_widget(f, chunks[2]);
        } else {
            self.render_help_widget(f, chunks[1]);
        }
    }

    /// Render the list of names
    fn render_names_widget(&self, f: &mut Frame, area: Rect) {
        let names = self.app.names();
        let timers = self.app.per_person_timers();
        let current_idx = self.app.current_person_index();

        // create list items with timer info
        let items: Vec<ListItem> = names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let timer_text = if timers[i] >= Duration::from_secs(5) {
                    format!(" ({})", format_duration(timers[i]))
                } else {
                    String::new()
                };

                let content = format!("{}:  {}{}", i + 1, name, timer_text);

                // highlight current person
                if i == current_idx {
                    ListItem::new(content)
                        .style(Style::default().bg(Color::Yellow).fg(Color::Black))
                } else {
                    ListItem::new(content)
                }
            })
            .collect();

        // create the list widget
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.app.config().title.as_str()),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        // create list state to track selection
        let mut state = ListState::default();
        state.select(Some(current_idx));

        f.render_stateful_widget(list, area, &mut state);
    }

    /// Render the timer widget with adaptive colors for light/dark backgrounds
    fn render_timer_widget(&self, f: &mut Frame, area: Rect) {
        let remaining = self.app.remaining_time();
        let total = self.app.config().duration;
        let is_dark = self.app.is_dark_background();

        // calculate progress (0.0 to 1.0)
        let progress = if total.as_secs() > 0 {
            // gauge decreases: start full (1.0) and goes to empty (0.0)
            remaining.as_secs() as f64 / total.as_secs() as f64
        } else {
            0.0
        };

        // choose icon based on remaining time
        let icon = if remaining.as_secs() > 180 {
            "⏳"
        } else {
            "⌛"
        };

        // Choose text color based on terminal background
        // For dark backgrounds: use light text (white)
        // For light backgrounds: use dark text (black/dark gray) for contrast
        let text_color = if is_dark { Color::White } else { Color::Black };

        // Create timer display with background for better visibility
        // The background ensures text is readable when gauge passes over it
        let text_bg = if is_dark {
            Color::Rgb(40, 40, 40) // Dark background for light text
        } else {
            Color::Rgb(240, 240, 240) // Light background for dark text
        };

        let timer_text = Span::styled(
            format!("{} {} left", icon, format_duration(remaining)),
            Style::default()
                .fg(text_color)
                .bg(text_bg)
                .add_modifier(Modifier::BOLD),
        );

        // create gauge color gradient style based on remaining time
        let gauge_style = if progress > 0.75 {
            // 75-100%: Bright green (plenty of time)
            Style::default().fg(Color::Rgb(34, 197, 94)) //Green-500
        } else if progress > 0.5 {
            // 50-75%: Light green
            Style::default().fg(Color::Rgb(132, 204, 22)) // Lime-500
        } else if progress > 0.35 {
            // 35-50%: Yellow-green
            Style::default().fg(Color::Rgb(163, 163, 0)) // Yellow-green mix
        } else if progress > 0.25 {
            // 25-35%: Yellow (caution)
            Style::default().fg(Color::Rgb(234, 179, 8)) // Yellow-500
        } else if progress > 0.15 {
            // 15-25%: Orange (warning)
            Style::default().fg(Color::Rgb(249, 115, 22)) // Orange-500
        } else if progress > 0.05 {
            // 5-15%: Red-orange (urgent)
            Style::default().fg(Color::Rgb(239, 68, 68)) // Red-500
        } else {
            // 0-5%: Bright red (critical)
            Style::default().fg(Color::Rgb(220, 38, 38)) // Red-600
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(gauge_style)
            .percent((progress * 100.0) as u16)
            .label(timer_text);

        f.render_widget(gauge, area);
    }

    /// Render the help widget
    fn render_help_widget(&self, f: &mut Frame, area: Rect) {
        let help_text = if self.app.config().hide_timer {
            "<Ctrl+N> Reshuffle names | <Tab/↓> Next | <↑> Previous | <Q> Quit"
        } else {
            "<Ctrl+R> Reset timer | <Ctrl+N> Reshuffle names | <Tab/↓> Next | <↑> Previous | <Q> Quit"
        };

        let paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    }
}

/// Format duration for display
fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;

    if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
