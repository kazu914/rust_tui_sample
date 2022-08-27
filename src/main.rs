use std::{io, usize};

use unicode_width::UnicodeWidthStr;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

enum Mode {
    Normal,
    Insert,
}

struct App {
    selected_list_index: Option<usize>,
    items: Vec<String>,
    input: String,
    mode: Mode,
}

impl App {
    fn enter_insert_mode(&mut self) {
        self.mode = Mode::Insert;
    }
    fn enter_normal_mode(&mut self) {
        self.mode = Mode::Normal;
    }
    fn select_next(&mut self) {
        match self.selected_list_index {
            Some(n) => {
                if n != self.items.len() - 1 {
                    self.selected_list_index = Some(n + 1);
                } else {
                    self.selected_list_index = Some(0);
                }
            }
            None => {}
        }
    }
    fn select_previous(&mut self) {
        match self.selected_list_index {
            Some(n) => {
                if n > 0 {
                    self.selected_list_index = Some(n - 1);
                } else {
                    self.selected_list_index = Some(self.items.len() - 1);
                }
            }
            None => {}
        }
    }
    fn push_input_to_items(&mut self) {
        self.items.push(self.input.drain(..).collect());
        if self.selected_list_index == None {
            self.selected_list_index = Some(0)
        }
        self.enter_normal_mode();
    }
    fn delete_selected_item(&mut self) {
        match self.selected_list_index {
            Some(n) => {
                self.items.remove(n);
                if self.items.is_empty() {
                    self.selected_list_index = None
                } else if self.items.len() == n {
                    self.select_previous();
                }
            }
            None => {}
        }
    }
}

impl Default for App {
    fn default() -> Self {
        App {
            selected_list_index: Some(1),
            items: vec!["aaa".to_string(), "bbb".to_string(), "ccc".to_string()],
            input: String::new(),
            mode: Mode::Normal,
        }
    }
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();

    loop {
        terminal.draw(|f| {
            ui(f, &app);
        })?;

        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            match app.mode {
                Mode::Insert => match (code, modifiers) {
                    (KeyCode::Esc, KeyModifiers::NONE) => {
                        app.enter_normal_mode();
                    }
                    (KeyCode::Enter, KeyModifiers::NONE) => {
                        app.push_input_to_items();
                    }
                    (KeyCode::Char(c), KeyModifiers::NONE) => {
                        app.input.push(c);
                    }
                    (KeyCode::Backspace, KeyModifiers::NONE) => {
                        app.input.pop();
                    }
                    _ => {}
                },
                Mode::Normal => match (code, modifiers) {
                    (KeyCode::Esc, KeyModifiers::NONE) => {
                        disable_raw_mode()?;
                        execute!(
                            terminal.backend_mut(),
                            LeaveAlternateScreen,
                            DisableMouseCapture
                        )?;

                        terminal.show_cursor()?;
                        return Ok(());
                    }
                    (KeyCode::Char('i'), KeyModifiers::NONE) => {
                        app.enter_insert_mode();
                    }
                    (KeyCode::Char('j'), KeyModifiers::NONE) => {
                        app.select_next();
                    }
                    (KeyCode::Char('k'), KeyModifiers::NONE) => {
                        app.select_previous();
                    }
                    (KeyCode::Char('d'), KeyModifiers::NONE) => {
                        app.delete_selected_item();
                    }
                    _ => {}
                },
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(f.size());

    let input = Paragraph::new(app.input.as_ref())
        .style(match app.mode {
            Mode::Normal => Style::default(),
            Mode::Insert => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[0]);
    let block2 = Block::default().title("block2").borders(Borders::ALL);
    f.render_widget(block2, chunks[2]);

    let items2 = app
        .items
        .iter()
        .map(|item| ListItem::new(item.to_string()))
        .collect::<Vec<ListItem>>();

    let list = List::new(items2)
        .block(Block::default().title("List").borders(Borders::ALL))
        .style(match app.mode {
            Mode::Insert => Style::default(),
            Mode::Normal => Style::default().fg(Color::Yellow),
        })
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>");
    let mut state = ListState::default();
    state.select(app.selected_list_index);
    f.render_stateful_widget(list, chunks[1], &mut state);

    match app.mode {
        Mode::Normal => {}
        Mode::Insert => f.set_cursor(chunks[0].x +  app.input.width_cjk() as u16 + 1, chunks[0].y + 1),
    }
}
