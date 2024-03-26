use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use homedir::get_my_home;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use reqwest::Client;
use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::constants::{ANTHROPIC_VERSION, MODELS};
use crate::request::Message;
use crate::request::MessageRequest;
use crate::response::MessageResponse;

#[allow(dead_code)]
mod constants;
#[allow(dead_code)]
mod parse_markup;
mod request;
mod response;

async fn send_message(
    prompt: &str,
    mut history: Vec<Message>,
    client: &Client,
    api_key: &str,
    model: &str,
) -> Result<Vec<Message>> {
    let message = Message::new("user", prompt);
    history.push(message);

    let body = MessageRequest::new(model.to_owned(), 1024, history.clone());

    let res = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("content-type", "application/json")
        .header("anthropic-version", ANTHROPIC_VERSION)
        .json(&body)
        .send()
        .await?
        .json::<MessageResponse>()
        .await?;

    if let Ok(mut file) = OpenOptions::new().append(true).open("kaite.log") {
        let content = res.content.clone();
        if let Some(respons) = content.unwrap_or_default().iter().last() {
            let _ = file.write_fmt(format_args!("assistant -> {}\n", respons.text));
        }
    }
    history.push(res.try_into()?);
    Ok(history)
}

#[derive(Debug, Default)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}

#[derive(Debug)]
struct App {
    input: Input,
    input_mode: InputMode,
    history: Vec<Message>,
    client: Client,
    api_key: String,
    scroll: u16,
    model: usize,
}

impl App {
    fn new(api_key: &str) -> Self {
        Self {
            input: Input::default(),
            input_mode: InputMode::Normal,
            history: vec![],
            client: Client::new(),
            api_key: api_key.to_string(),
            scroll: 0,
            model: 0,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Read .kaite.env from home dir if it exists
    if let Some(home_dir) = get_my_home()
        .ok()
        .flatten()
        .map(|path| path.join(".kaite.env"))
        .filter(|path| path.exists())
    {
        dotenv::from_path(home_dir.as_path())?;
    }

    let _ = File::create("kaite.log");

    let api_key = std::env::var("API_KEY")
        .context("The API_KEY environment variable needs to be set either temporarily through the terminal or in a .kaite.env file in your home directory!")?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(&api_key);
    let res = run_app(&mut terminal, app).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        println!("Failed with err {e:?}");
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('i') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('j') => {
                        app.scroll += 1;
                    }
                    KeyCode::Char('k') => {
                        if app.scroll > 0 {
                            app.scroll -= 1;
                        }
                    }
                    KeyCode::Char('m') => {
                        app.model = (app.model + 1) % MODELS.len();
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        if app.input.value().is_empty() {
                            continue;
                        }

                        let input = app.input.value().to_owned();
                        app.input.reset();

                        if let Ok(mut file) = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("kaite.log")
                        {
                            let _ = file.write_fmt(format_args!("user -> {}\n", input));
                        }

                        let model = MODELS[app.model];
                        let history =
                            send_message(&input, app.history, &app.client, &app.api_key, &model)
                                .await?;

                        app.input.reset();
                        app = App { history, ..app };
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {
                        app.input.handle_event(&Event::Key(key));
                    }
                },
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let width = chunks[0].width.max(3) - 3;
    let scroll = app.input.visual_scroll(width as usize);
    let input = Paragraph::new(app.input.value())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Magenta),
        })
        .scroll((0, scroll as u16))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("\u{e795} Prompt")
                .title_style(Style::default().bold().magenta()),
        );
    f.render_widget(input, chunks[1]);

    if let InputMode::Editing = app.input_mode {
        f.set_cursor(
            chunks[1].x + ((app.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
            chunks[1].y + 1,
        );
    }

    let messages: Vec<Line> = app.history.iter().fold(vec![], |mut acc, msg| {
        let color = if msg.role.to_lowercase() == "user" {
            Color::Magenta
        } else {
            Color::LightGreen
        };

        let speaker = vec![Span::default()
            .content(format!("{}: ", msg.role))
            .style(Style::default().italic().fg(color))];
        acc.push(Line::default().spans(speaker));

        let mut lines = match msg.role.as_str() {
            "user" => {
                let spans = vec![Span::default().content(msg.content.clone())];
                vec![Line::default().spans(spans)]
            }
            "assistant" => msg
                .content
                .split("\n")
                .flat_map(parse_markup::convert)
                .collect::<Vec<Line>>(),
            _ => unimplemented!(),
        };
        lines.push(Line::default().spans(vec![Span::default()]));

        acc.append(&mut lines);
        acc
    });

    let model = MODELS[app.model];
    let title = format!("\u{eb26} Conversation - {}", model);

    let paragraph = Paragraph::new(messages)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: true })
        .scroll((app.scroll, 0));

    f.render_widget(paragraph, chunks[2]);
}
