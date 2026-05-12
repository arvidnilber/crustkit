use std::{
    fs,
    io::{self, Stdout},
    path::{Path, PathBuf},
    time::Duration,
};

use chrono::Utc;
use clap::Parser;
use color_eyre::{Result, eyre::Context};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Cell, List, ListItem, ListState, Paragraph, Row, Table, TableState,
    },
};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use url::Url;

const BASE_URL: &str = "https://api.anthropic.com";

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Export claude.ai conversations through Anthropic Compliance API"
)]
struct Args {
    #[arg(long, env = "ANTHROPIC_COMPLIANCE_ACCESS_KEY")]
    api_key: String,

    #[arg(long, default_value = BASE_URL)]
    base_url: String,

    #[arg(long = "user-id", required = true)]
    user_ids: Vec<String>,

    #[arg(long)]
    project_id: Option<String>,

    #[arg(long, default_value = "anthropic-conversation-export")]
    output_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Project {
    id: String,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Chat {
    id: String,
    name: String,
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Page<T> {
    data: Vec<T>,
    #[serde(default)]
    has_more: bool,
    #[serde(default)]
    last_id: Option<String>,
    #[serde(default)]
    next_page: Option<String>,
}

#[derive(Debug)]
struct App {
    args: Args,
    client: ComplianceClient,
    projects: Vec<Project>,
    selected_project: usize,
    chats: Vec<Chat>,
    table_state: TableState,
    status: String,
    busy: bool,
    exit: bool,
}

impl App {
    fn new(args: Args) -> Result<Self> {
        let client = ComplianceClient::new(args.base_url.clone(), args.api_key.clone())?;
        let mut projects = client.list_projects()?;
        projects.sort_by_key(|project| project.name.to_lowercase());
        projects.insert(
            0,
            Project {
                id: "all".to_string(),
                name: "All projects".to_string(),
            },
        );

        let selected_project = args
            .project_id
            .as_ref()
            .and_then(|id| projects.iter().position(|project| &project.id == id))
            .unwrap_or(0);

        let mut app = Self {
            args,
            client,
            projects,
            selected_project,
            chats: Vec::new(),
            table_state: TableState::default(),
            status: "Enter exports selected scope. Space changes project. q quits.".to_string(),
            busy: false,
            exit: false,
        };
        app.refresh_chats()?;
        Ok(app)
    }

    fn run(mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render(frame))?;

            if !event::poll(Duration::from_millis(200))? {
                continue;
            }

            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.handle_key(key.code)?;
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, code: KeyCode) -> Result<()> {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
            KeyCode::Down | KeyCode::Char('j') => self.next_project()?,
            KeyCode::Up | KeyCode::Char('k') => self.previous_project()?,
            KeyCode::Char(' ') => self.refresh_chats()?,
            KeyCode::Enter => self.export_selected_scope()?,
            _ => {}
        }
        Ok(())
    }

    fn next_project(&mut self) -> Result<()> {
        self.selected_project = (self.selected_project + 1) % self.projects.len();
        self.refresh_chats()
    }

    fn previous_project(&mut self) -> Result<()> {
        self.selected_project = if self.selected_project == 0 {
            self.projects.len() - 1
        } else {
            self.selected_project - 1
        };
        self.refresh_chats()
    }

    fn refresh_chats(&mut self) -> Result<()> {
        self.busy = true;
        let project_id = self.current_project_id();
        self.chats = self.client.list_chats(&self.args.user_ids, project_id)?;
        self.table_state.select_first();
        self.status = format!(
            "Loaded {} chats for {}",
            self.chats.len(),
            self.current_project().name
        );
        self.busy = false;
        Ok(())
    }

    fn export_selected_scope(&mut self) -> Result<()> {
        self.busy = true;
        let started = Utc::now();
        let project = self.current_project().clone();
        let root = self
            .args
            .output_dir
            .join(started.format("%Y-%m-%d_%H-%M-%S").to_string())
            .join(safe_path_part(&project.name));

        fs::create_dir_all(&root).wrap_err_with(|| format!("creating {}", root.display()))?;

        let manifest_path = root.join("_manifest.json");
        let mut exported = Vec::new();

        for chat in &self.chats {
            let messages = self.client.chat_messages(&chat.id)?;
            let filename = format!("{}_{}.json", safe_path_part(chat.name.as_str()), chat.id);
            let path = root.join(filename);
            let raw = json!({
                "chat": chat,
                "messages": messages,
            });
            write_json(&path, &raw)?;
            exported.push(json!({
                "id": chat.id,
                "name": chat.name,
                "project_id": chat.project_id,
                "path": path,
            }));
        }

        write_json(
            &manifest_path,
            &json!({
                "exported_at": started,
                "project": project,
                "user_ids": self.args.user_ids,
                "chat_count": exported.len(),
                "chats": exported,
            }),
        )?;

        self.status = format!("Exported {} chats to {}", self.chats.len(), root.display());
        self.busy = false;
        Ok(())
    }

    fn current_project(&self) -> &Project {
        &self.projects[self.selected_project]
    }

    fn current_project_id(&self) -> Option<&str> {
        let project = self.current_project();
        (project.id != "all").then_some(project.id.as_str())
    }

    fn render(&mut self, frame: &mut Frame) {
        let root = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .spacing(1)
        .split(frame.area());

        self.render_header(frame, root[0]);

        let compact = frame.area().width < 96;
        let body = if compact {
            Layout::vertical([Constraint::Length(12), Constraint::Fill(1)])
                .spacing(1)
                .split(root[1])
        } else {
            Layout::horizontal([Constraint::Percentage(36), Constraint::Percentage(64)])
                .spacing(2)
                .split(root[1])
        };

        self.render_projects(frame, body[0]);
        self.render_chats(frame, body[1]);
        self.render_footer(frame, root[2]);
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let title = Line::from(vec![
            Span::styled(" ◆ ", Style::new().cyan()),
            Span::styled("Anthropic Export", Style::new().bold()),
            Span::raw("  "),
            Span::styled("raw conversation JSON", Style::new().dim()),
        ]);

        frame.render_widget(
            Paragraph::new(title).block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Color::DarkGray),
            ),
            area,
        );
    }

    fn render_projects(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .projects
            .iter()
            .enumerate()
            .map(|(index, project)| {
                let radio = if index == self.selected_project {
                    "◉"
                } else {
                    "○"
                };
                let radio_style = if index == self.selected_project {
                    Style::new().green().bold()
                } else {
                    Style::new().fg(Color::DarkGray)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(radio, radio_style),
                    Span::raw("  "),
                    Span::raw(project.name.clone()),
                ]))
            })
            .collect();

        let mut state = ListState::default().with_selected(Some(self.selected_project));
        let list = List::new(items)
            .block(
                Block::bordered()
                    .title(" Project filter ")
                    .border_type(BorderType::Rounded)
                    .border_style(Color::Cyan),
            )
            .highlight_symbol("› ")
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED));

        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_chats(&mut self, frame: &mut Frame, area: Rect) {
        let narrow = area.width < 78;
        let header = if narrow {
            Row::new(["Chat", "Updated"]).bold()
        } else {
            Row::new(["Chat", "ID", "Updated"]).bold()
        };

        let rows = self.chats.iter().map(|chat| {
            let updated = chat.updated_at.as_deref().unwrap_or("-");
            if narrow {
                Row::new(vec![
                    Cell::from(chat.name.clone()),
                    Cell::from(updated.to_string()),
                ])
            } else {
                Row::new(vec![
                    Cell::from(chat.name.clone()),
                    Cell::from(chat.id.clone()),
                    Cell::from(updated.to_string()),
                ])
            }
        });

        let widths = if narrow {
            vec![Constraint::Percentage(65), Constraint::Percentage(35)]
        } else {
            vec![
                Constraint::Percentage(42),
                Constraint::Percentage(35),
                Constraint::Percentage(23),
            ]
        };

        let table = Table::new(rows, widths)
            .header(header.bottom_margin(1))
            .block(
                Block::bordered()
                    .title(" Conversations ")
                    .border_type(BorderType::Rounded)
                    .border_style(Color::DarkGray),
            )
            .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED));

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let mut status = self.status.clone();
        if self.busy {
            status = format!("{} ...", status);
        }

        let help = Line::from(vec![
            Span::styled("↑/↓ j/k", Style::new().bold()),
            Span::raw(" project  "),
            Span::styled("Space", Style::new().bold()),
            Span::raw(" refresh  "),
            Span::styled("Enter", Style::new().bold()),
            Span::raw(" export  "),
            Span::styled("q", Style::new().bold()),
            Span::raw(" quit  "),
            Span::styled(status, Style::new().dim()),
        ]);

        frame.render_widget(Paragraph::new(help), area);
    }
}

#[derive(Debug)]
struct ComplianceClient {
    http: Client,
    base_url: Url,
    api_key: String,
}

impl ComplianceClient {
    fn new(base_url: String, api_key: String) -> Result<Self> {
        Ok(Self {
            http: Client::builder()
                .user_agent("anthropic-export-tui/0.1.0")
                .build()?,
            base_url: Url::parse(&base_url)?,
            api_key,
        })
    }

    fn list_projects(&self) -> Result<Vec<Project>> {
        let mut projects = Vec::new();
        let mut page: Option<String> = None;

        loop {
            let mut url = self.url("/v1/compliance/apps/projects")?;
            if let Some(page) = &page {
                url.query_pairs_mut().append_pair("page", page);
            }
            let body: Page<Project> = self.get_json(url)?;
            projects.extend(body.data);
            match body.next_page {
                Some(next_page) => page = Some(next_page),
                None => break,
            }
        }

        Ok(projects)
    }

    fn list_chats(&self, user_ids: &[String], project_id: Option<&str>) -> Result<Vec<Chat>> {
        let mut chats = Vec::new();
        let mut after_id: Option<String> = None;

        loop {
            let mut url = self.url("/v1/compliance/apps/chats")?;
            {
                let mut query = url.query_pairs_mut();
                for user_id in user_ids {
                    query.append_pair("user_ids[]", user_id);
                }
                if let Some(project_id) = project_id {
                    query.append_pair("project_ids[]", project_id);
                }
                if let Some(after_id) = &after_id {
                    query.append_pair("after_id", after_id);
                }
            }

            let body: Page<Chat> = self.get_json(url)?;
            chats.extend(body.data);
            if body.has_more {
                after_id = body.last_id;
            } else {
                break;
            }
        }

        Ok(chats)
    }

    fn chat_messages(&self, chat_id: &str) -> Result<Value> {
        let path = format!("/v1/compliance/apps/chats/{chat_id}/messages");
        self.get_json(self.url(&path)?)
    }

    fn get_json<T: for<'de> Deserialize<'de>>(&self, url: Url) -> Result<T> {
        let response = self
            .http
            .get(url)
            .header("x-api-key", &self.api_key)
            .send()?
            .error_for_status()?;
        Ok(response.json()?)
    }

    fn url(&self, path: &str) -> Result<Url> {
        Ok(self.base_url.join(path.trim_start_matches('/'))?)
    }
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes).wrap_err_with(|| format!("writing {}", path.display()))
}

fn safe_path_part(input: &str) -> String {
    let sanitized = sanitize_filename::sanitize(input.trim());
    if sanitized.is_empty() {
        "untitled".to_string()
    } else {
        sanitized
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = App::new(args)?.run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
