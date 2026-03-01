use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Alignment,  // 👈 Исправлено: prelude вместо style
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

// === Типы меню ===

#[derive(Debug, Clone, Copy, PartialEq)]
enum MenuState {
    Closed,
    FileOpen,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MenuItem {
    FileNew,
    FileOpen,
    FileSave,
    FileExit,
}

struct Menu {
    state: MenuState,
    selected: usize,
}

impl Menu {
    fn new() -> Self {
        Self { state: MenuState::Closed, selected: 0 }
    }

    fn is_open(&self) -> bool {
        matches!(self.state, MenuState::FileOpen)
    }

    fn toggle(&mut self) {
        self.state = if self.is_open() { MenuState::Closed } else { MenuState::FileOpen };
        self.selected = 0;
    }
    fn close(&mut self) {
        self.state = MenuState::Closed;
    }

    fn navigate(&mut self, up: bool) {
        let items = Self::items();
        if up {
            self.selected = self.selected.saturating_sub(1);
        } else {
            self.selected = (self.selected + 1).min(items.len() - 1);
        }
    }

    fn items() -> &'static [&'static str] {
        &["New", "Open", "Save", "Exit"]
    }

    fn selected_item(&self) -> Option<MenuItem> {
        if !self.is_open() { return None; }
        match self.selected {
            0 => Some(MenuItem::FileNew),
            1 => Some(MenuItem::FileOpen),
            2 => Some(MenuItem::FileSave),
            3 => Some(MenuItem::FileExit),
            _ => None,
        }
    }
}

// === Приложение ===

struct App {
    running: bool,
    filename: Option<String>,
    menu: Menu,
    buffer: String,
    modified: bool,
    confirm_quit: bool,
}

impl App {
    fn new(filename: Option<String>) -> Self {
        Self {
            running: true,
            filename,
            menu: Menu::new(),
            buffer: String::new(),
            modified: false,
            confirm_quit: false,        }
    }

    fn handle_key(&mut self, key: event::KeyEvent) -> Result<()> {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        // Подтверждение выхода
        if self.confirm_quit {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if let Some(ref name) = self.filename {
                        std::fs::write(name, &self.buffer)?;
                    }
                    self.running = false;
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.running = false;
                }
                KeyCode::Esc => {
                    self.confirm_quit = false;
                }
                _ => {}
            }
            return Ok(());
        }

        // Меню
        if self.menu.is_open() {
            match key.code {
                KeyCode::Esc => self.menu.close(),
                KeyCode::Up | KeyCode::Char('k') => self.menu.navigate(true),
                KeyCode::Down | KeyCode::Char('j') => self.menu.navigate(false),
                KeyCode::Enter => {
                    if let Some(item) = self.menu.selected_item() {
                        self.exec_item(item)?;
                        self.menu.close();
                    }
                }
                _ => {}
            }
            return Ok(());
        }

        // Команды с Ctrl
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('q') => self.confirm_quit = true,
                KeyCode::Char('s') => {                    if let Some(ref name) = self.filename {
                        std::fs::write(name, &self.buffer)?;
                        self.modified = false;
                    }
                }
                KeyCode::Char('o') => {
                    self.buffer = "// TODO: Open file dialog\n".into();
                    self.modified = false;
                }
                KeyCode::Char('n') => {
                    self.buffer.clear();
                    self.filename = None;
                    self.modified = false;
                }
                _ => {}
            }
            return Ok(());
        }

        // Обычный ввод текста
        match key.code {
            KeyCode::F(9) => self.menu.toggle(),
            KeyCode::Enter => self.buffer.push('\n'),
            KeyCode::Tab => self.buffer.push_str("    "),
            KeyCode::Char(c) => {
                self.buffer.push(c);
                self.modified = true;
            }
            KeyCode::Backspace => {
                self.buffer.pop();
                self.modified = true;
            }
            _ => {}
        }
        Ok(())
    }

    fn exec_item(&mut self, item: MenuItem) -> Result<()> {
        match item {
            MenuItem::FileNew => {
                self.buffer.clear();
                self.filename = None;
                self.modified = false;
            }
            MenuItem::FileOpen => {
                self.buffer = "// TODO: Open file dialog\n".into();
                self.modified = false;
            }
            MenuItem::FileSave => {
                if let Some(ref name) = self.filename {                    std::fs::write(name, &self.buffer)?;
                    self.modified = false;
                }
            }
            MenuItem::FileExit => {
                self.confirm_quit = true;
            }
        }
        Ok(())
    }
}

// === Терминал ===

fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    Ok(Terminal::new(CrosstermBackend::new(io::stdout()))?)
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

// === Отрисовка меню ===

fn render_menu(f: &mut Frame, menu: &Menu, area: Rect) {
    if !menu.is_open() {
        return;
    }

    let w = 20u16;
    let h = 6u16;
    let popup = Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    );

    let items: Vec<ListItem> = Menu::items()
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let style = if i == menu.selected {
                Style::default().bg(Color::Blue).fg(Color::White).bold()
            } else {
                Style::default()            };
            ListItem::new(format!(" {}", text)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title(" File ").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

    f.render_widget(ratatui::widgets::Clear, popup);
    f.render_widget(list, popup);
}

// === Отрисовка подтверждения выхода ===

fn render_quit_confirm(f: &mut Frame, area: Rect) {
    let w = 45u16;
    let h = 7u16;
    let popup = Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    );

    let text = Paragraph::new("Save changes before quitting?\n\n[y] Yes  [n] No  [Esc] Cancel")
        .block(Block::default().title(" Confirm Exit ").borders(Borders::ALL))
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .alignment(Alignment::Center);

    f.render_widget(ratatui::widgets::Clear, popup);
    f.render_widget(text, popup);
}

// === Главный цикл отрисовки ===

fn draw_ui(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &App) -> Result<()> {
    terminal.draw(|f| {
        let size = f.size();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(size);
        // Header
        let title = Paragraph::new(" iRm — Rust CLI IDE ")
            .style(Style::default().fg(Color::Cyan).bold())
            .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(title, chunks[0]);

        // Editor
        let content = if app.buffer.is_empty() {
            if let Some(ref name) = app.filename {
                format!("📄 Editing: {}", name)
            } else {
                "Welcome! [F9] Menu, [Ctrl+Q] Quit".into()
            }
        } else {
            app.buffer.clone()
        };
        let editor = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title(" Editor "));
        f.render_widget(editor, chunks[1]);

        // Status
        let modified_indicator = if app.modified { " [+]" } else { "" };
        let status_text = if app.confirm_quit {
            " Save? [y] Yes  [n] No  [Esc] Cancel "
        } else if app.menu.is_open() {
            " [↑↓] Nav | [Enter] Select | [Esc] Close "
        } else {
            &format!(" [F9] Menu | [Ctrl+Q] Quit | [Ctrl+S] Save{} ", modified_indicator)
        };
        let status_bar = Paragraph::new(status_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::TOP));
        f.render_widget(status_bar, chunks[2]);

        // Overlays
        render_menu(f, &app.menu, size);
        if app.confirm_quit {
            render_quit_confirm(f, size);
        }
    })?;
    Ok(())
}

// === main ===

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).cloned();

    let mut app = App::new(filename);    let mut terminal = init_terminal()?;

    while app.running {
        draw_ui(&mut terminal, &app)?;
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key)?;
            }
        }
    }

    restore_terminal()?;
    println!("👋 iRm exited.");
    Ok(())
}
