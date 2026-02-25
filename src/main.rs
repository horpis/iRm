use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},  // 👈 Добавили Stylize!
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;

// Состояние приложения
struct App {
    running: bool,
    filename: Option<String>,
}

impl App {
    fn new(filename: Option<String>) -> Self {
        Self { running: true, filename }
    }

    fn handle_key(&mut self, key: event::KeyEvent) -> Result<()> {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.running = false,
            _ => {}
        }
        Ok(())
    }
}

// Инициализация терминала
fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    Ok(Terminal::new(CrosstermBackend::new(io::stdout()))?)
}

// Восстановление терминала
fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())}

// Главный цикл отрисовки
fn draw_ui(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &App) -> Result<()> {
    terminal.draw(|f| {
        // 👈 Исправлено: size() вместо area()
        let size = f.size();
        
        // Вертикальный лейаут: заголовок + контент + футер
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1),  // Header
                Constraint::Min(0),     // Content
                Constraint::Length(1),  // Status bar
            ])
            .split(size);

        // Заголовок
        let title = Paragraph::new(" iRm — Rust CLI IDE ")
            .style(Style::default().fg(Color::Cyan).bold())  // 👈 bold() работает благодаря импорту Stylize
            .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(title, chunks[0]);

        // Область контента
        let content = if let Some(ref name) = app.filename {
            Paragraph::new(format!("Editing: {}", name))
        } else {
            Paragraph::new("Welcome! Open a file with: irm <filename>")
        }
        .block(Block::default().borders(Borders::ALL).title(" Editor "));
        f.render_widget(content, chunks[1]);

        // Статус-бар
        let status = Paragraph::new(" [q] Quit | [Ctrl+C] Force Exit ")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::TOP));
        f.render_widget(status, chunks[2]);
    })?;
    Ok(())
}

// Точка входа
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let filename = if args.len() > 1 { Some(args[1].clone()) } else { None };

    let mut app = App::new(filename);
    let mut terminal = init_terminal()?;    
    while app.running {
        draw_ui(&mut terminal, &app)?;
        
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key)?;
            }
        }
    }

    restore_terminal()?;
    println!("👋 iRm exited gracefully.");
    Ok(())
}
