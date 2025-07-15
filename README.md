# nextup

A Rust terminal application that randomizes a list of names. Useful for daily standup meetings.

## Features
- **Randomized name display**: Shuffle team members for a fair standup order
- **Per-person timers**: Track how long each person speaks (displayed after 5 seconds)
- **Meeting timer**: Visual progress bar showing remaining meeting time
- **Keyboard shortcuts**: Easy navigation and control
- **Configurable**: Customizable meeting duration, title, and timer visibility


## Prerequisites
- Rust 1.70 or later
- A terminal that supports Unicode (for timer icons)


## Installation
1. Clone or create the project directory
2. A team.txt file is included with example names (feel free to replace with your team):
   ```
   Stan Marsh
   Kyle Broflovski
   Kenny McCormick
   Eric Cartman
   Heidi Turner
   Butters Stotch
   ```


## Building
```bash
# Build the project
cargo build --release

# The executable will be at target/release/nextup
```


## Usage

### Basic Usage
```bash
# Run with default settings
./target/release/nextup

# Or use cargo run during development
cargo run
```

### Command Line Options
```bash
# Customize the meeting
./target/release/nextup \
  --title "Sprint Review" \
  --duration 20 \
  --names "sprint-team.txt"

# Hide the timer
./target/release/nextup --hide-timer

# See all options
./target/release/nextup --help
```

### Keyboard Shortcuts
| Key | Action |
|-----|--------|
| `Tab` or `↓` | Move to next person |
| `↑` | Move to previous person |
| `Ctrl+N` | Reshuffle names and reset timers |
| `Ctrl+R` | Reset timer and per-person timers |
| `Q` or `Ctrl+C` | Quit |


## Configuration Options
- `--title`: Set the window title (default: "Team daily standup")
- `--names`: Path to names file (default: "team.txt")
- `--duration`: Meeting duration in minutes (default: 15)
- `--hide-timer`: Hide the timer widget


## Development
```bash
# Run in development mode with auto-reload
cargo watch -x run

# Run with specific arguments
cargo run -- --duration 10 --hide-timer

# Format code
cargo fmt

# Check for issues
cargo clippy
```

## File Structure
```
src/
├── main.rs          # Entry point and CLI parsing
├── app.rs           # Main application logic and state
├── config.rs        # Configuration structure
├── error.rs         # Error types and handling
└── ui.rs            # Ratatui UI components
```


## Tips
1. **Terminal Size**: Make your terminal text large (`Ctrl+Plus` in most terminals) for better visibility
2. **Unicode Support**: Ensure your terminal supports Unicode for timer icons
3. **Performance**: The app updates every 500ms for smooth timer display
4. **Team File**: Keep your `team.txt` file in the same directory as the executable


## Troubleshooting
- **"No names found"**: Check that your names file exists and contains names
- **Unicode issues**: Use a modern terminal that supports Unicode
- **Timer not updating**: Check that your terminal supports the [gauge widget](https://ratatui.rs/examples/widgets/gauge/)


## Architecture Callouts
- **Error Handling**: Uses Rust's [`Result`](https://doc.rust-lang.org/std/result/index.html) type with error propagation
- **Configuration**: Uses [`clap`](https://github.com/clap-rs/clap) for CLI parsing
- **Async Runtime**: Uses [`tokio`](https://github.com/tokio-rs/tokio) for async operations
- **Composable TUI**: Uses [`ratatui`](https://github.com/ratatui/ratatui) for terminal user interface (TUI)
- **Memory Safety**: Rust's ownership system prevents memory leaks


## Future Enhancements
- Configuration file support
- Custom timer icons
- Sound notifications
- Export meeting stats
- Integration with calendar systems