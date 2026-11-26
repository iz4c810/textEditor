use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use ratatui_textarea::{Input, TextArea};
use std::{env, fs, io, path::PathBuf};

struct App<'a> {
    textarea: TextArea<'a>,
    file_path: Option<PathBuf>,
    status_message: String,
    modified: bool,
}

impl<'a> App<'a> {
    fn new(file_path: Option<PathBuf>) -> io::Result<Self> {
        let mut textarea = if let Some(ref path) = file_path {
            if path.exists() {
                let content = fs::read_to_string(path)?;
                let lines: Vec<String> = content.lines().map(String::from).collect();
                TextArea::new(lines)
            } else {
                TextArea::default()
            }
        } else {
            TextArea::default()
        };

        // Configure visual styling for the editor
        textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
        textarea.set_cursor_line_style(Style::default().bg(Color::Rgb(30, 30, 40)));
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(
                    file_path
                        .as_ref()
                        .and_then(|p| p.to_str())
                        .unwrap_or("[No Name]"),
                ),
        );

        Ok(Self {
            textarea,
            file_path,
            status_message: String::from("Ctrl+S: Save | Ctrl+Q: Quit"),
            modified: false,
        })
    }

    fn save(&mut self) {
        let path = self
            .file_path
            .get_or_insert_with(|| PathBuf::from("untitled.txt"));

        let content = self.textarea.lines().join("\n");
        match fs::write(&path, content) {
            Ok(_) => {
                self.modified = false;
                self.status_message = format!("Saved to {}", path.display());
            }
            Err(e) => {
                self.status_message = format!("Error saving file: {}", e);
            }
        }
    }
}

fn main() -> io::Result<()> {
    // Parse optional filename from CLI arguments
    let file_path = env::args().nth(1).map(PathBuf::from);
    let mut app = App::new(file_path)?;

    // Initialize terminal in raw mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, &mut app);

    // Always restore terminal state on exit
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Application error: {:?}", err);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),    // Main editor area
                    Constraint::Length(1), // Bottom status bar
                ])
                .split(frame.area());

            // Render text editor widget
            frame.render_widget(&app.textarea, chunks[0]);

            // Render status bar
            let (row, col) = app.textarea.cursor();
            let modified_flag = if app.modified { " [+]" } else { "" };
            let status_text = format!(
                "{}{}  |  Ln {}, Col {}",
                app.status_message,
                modified_flag,
                row + 1,
                col + 1
            );

            let status_bar = Paragraph::new(status_text).style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );
            frame.render_widget(status_bar, chunks[1]);
        })?;

        // Handle keyboard input
        if let Event::Key(key) = event::read()? {
            match (key.modifiers, key.code) {
                (KeyModifiers::CONTROL, KeyCode::Char('q')) => break,
                (KeyModifiers::CONTROL, KeyCode::Char('s')) => app.save(),
                _ => {
                    // Forward all other key events to ratatui-textarea
                    if app.textarea.input(Input::from(key)) {
                        app.modified = true;
                    }
                }
            }
        }
    }
    Ok(())
}
