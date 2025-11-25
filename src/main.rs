use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::fs;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}

#[derive(Debug, Clone, PartialEq)]
enum AppState {
    Confirmation, // Main screen - shows file status and menu
    EnvSetup,     // Interactive form for .env setup
    Installing,
    Success,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
enum MenuSelection {
    Proceed,        // Proceed with installation (only if all files exist)
    GenerateEnv,    // Generate .env file with form
    GenerateConfig, // Generate config.yaml from template
    Cancel,         // Exit
}

#[derive(Debug, Clone)]
struct FormData {
    openai_api_key: String,
    generation_model: String,
    host_port: String,
    ai_service_port: String,
    current_field: usize,
    editing: bool,
    error_message: String,
}

impl FormData {
    fn new() -> Self {
        Self {
            openai_api_key: String::new(),
            generation_model: "gpt-4o-mini".to_string(),
            host_port: "3000".to_string(),
            ai_service_port: "5555".to_string(),
            current_field: 0,
            editing: false,
            error_message: String::new(),
        }
    }

    fn validate(&mut self) -> bool {
        if self.openai_api_key.trim().is_empty() {
            self.error_message = "OpenAI API Key is required!".to_string();
            return false;
        }

        if !self.openai_api_key.starts_with("sk-") {
            self.error_message =
                "Invalid OpenAI API Key format (should start with 'sk-')".to_string();
            return false;
        }

        self.error_message.clear();
        true
    }

    fn get_current_value_mut(&mut self) -> &mut String {
        match self.current_field {
            0 => &mut self.openai_api_key,
            1 => &mut self.generation_model,
            2 => &mut self.host_port,
            3 => &mut self.ai_service_port,
            _ => &mut self.openai_api_key,
        }
    }
}

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App {
    running: bool,
    state: AppState,
    logs: Vec<String>,
    progress: f64,
    current_service: String,
    total_services: usize,
    completed_services: usize,
    // File detection
    env_exists: bool,
    config_exists: bool,
    // Form data
    form_data: FormData,
    // Menu selection
    menu_selection: MenuSelection,
}

impl App {
    pub fn new() -> Self {
        // Check if required files exist
        let env_exists = Self::find_file(".env");
        let config_exists = Self::find_file("config.yaml");

        // Always start at Confirmation screen
        let initial_state = AppState::Confirmation;

        // Determine initial menu selection based on what's missing
        let initial_menu = if !env_exists {
            MenuSelection::GenerateEnv
        } else if !config_exists {
            MenuSelection::GenerateConfig
        } else {
            MenuSelection::Proceed
        };

        Self {
            running: true,
            state: initial_state,
            logs: Vec::new(),
            progress: 0.0,
            current_service: String::new(),
            total_services: 4, // analytics-service, qdrant, northwind-db, analytics-ui
            completed_services: 0,
            env_exists,
            config_exists,
            form_data: FormData::new(),
            menu_selection: initial_menu,
        }
    }

    /// Find a file in current directory or parent directories
    fn find_file(filename: &str) -> bool {
        if std::path::Path::new(filename).exists() {
            return true;
        }

        let parent_path = format!("../../{}", filename);
        if std::path::Path::new(&parent_path).exists() {
            return true;
        }

        false
    }

    /// Get the project root directory
    fn get_project_root() -> std::path::PathBuf {
        if std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.contains("target")))
            .unwrap_or(false)
        {
            std::env::current_dir()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .to_path_buf()
        } else {
            std::env::current_dir().unwrap()
        }
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            match &self.state {
                AppState::Confirmation => {
                    if let Some(action) = self.handle_confirmation_events()? {
                        match action {
                            MenuSelection::Proceed => {
                                if self.env_exists && self.config_exists {
                                    self.state = AppState::Installing;
                                    self.logs
                                        .push("🚀 Starting Analytics installation...".to_string());

                                    let result = self.run_docker_compose().await;

                                    match result {
                                        Ok(_) => {
                                            self.state = AppState::Success;
                                            self.progress = 100.0;
                                        }
                                        Err(e) => {
                                            self.state = AppState::Error(format!(
                                                "Installation failed: {}",
                                                e
                                            ));
                                        }
                                    }
                                }
                            }
                            MenuSelection::GenerateEnv => {
                                self.state = AppState::EnvSetup;
                            }
                            MenuSelection::GenerateConfig => {
                                if let Err(e) = self.generate_config_yaml() {
                                    self.state = AppState::Error(format!(
                                        "Failed to generate config.yaml: {}",
                                        e
                                    ));
                                } else {
                                    self.config_exists = true;
                                    // Update menu selection
                                    if !self.env_exists {
                                        self.menu_selection = MenuSelection::GenerateEnv;
                                    } else {
                                        self.menu_selection = MenuSelection::Proceed;
                                    }
                                }
                            }
                            MenuSelection::Cancel => {
                                self.running = false;
                            }
                        }
                    }
                }
                AppState::EnvSetup => {
                    if let Some(proceed) = self.handle_form_events()? {
                        if proceed {
                            if let Err(e) = self.generate_env_file() {
                                self.state =
                                    AppState::Error(format!("Failed to generate .env: {}", e));
                            } else {
                                self.env_exists = true;
                                self.state = AppState::Confirmation;
                                // Update menu selection
                                if !self.config_exists {
                                    self.menu_selection = MenuSelection::GenerateConfig;
                                } else {
                                    self.menu_selection = MenuSelection::Proceed;
                                }
                            }
                        } else {
                            self.state = AppState::Confirmation;
                        }
                    }
                }
                AppState::Installing => {
                    if event::poll(std::time::Duration::from_millis(100))? {
                        if let Event::Key(key) = event::read()? {
                            if key.kind == KeyEventKind::Press {
                                if let KeyCode::Char('c') = key.code {
                                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                                        self.running = false;
                                    }
                                }
                            }
                        }
                    }
                }
                AppState::Success | AppState::Error(_) => {
                    if event::poll(std::time::Duration::from_millis(100))? {
                        if let Event::Key(key) = event::read()? {
                            if key.kind == KeyEventKind::Press {
                                if let KeyCode::Char('c') = key.code {
                                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                                        self.running = false;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_confirmation_events(&mut self) -> Result<Option<MenuSelection>> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Up => {
                            self.menu_selection = match self.menu_selection {
                                MenuSelection::Proceed => {
                                    if !self.config_exists {
                                        MenuSelection::GenerateConfig
                                    } else if !self.env_exists {
                                        MenuSelection::GenerateEnv
                                    } else {
                                        MenuSelection::Cancel
                                    }
                                }
                                MenuSelection::GenerateEnv => MenuSelection::Cancel,
                                MenuSelection::GenerateConfig => {
                                    if !self.env_exists {
                                        MenuSelection::GenerateEnv
                                    } else {
                                        MenuSelection::Cancel
                                    }
                                }
                                MenuSelection::Cancel => {
                                    if self.env_exists && self.config_exists {
                                        MenuSelection::Proceed
                                    } else if !self.config_exists {
                                        MenuSelection::GenerateConfig
                                    } else {
                                        MenuSelection::GenerateEnv
                                    }
                                }
                            };
                        }
                        KeyCode::Down | KeyCode::Tab => {
                            self.menu_selection = match self.menu_selection {
                                MenuSelection::Proceed => MenuSelection::Cancel,
                                MenuSelection::GenerateEnv => {
                                    if !self.config_exists {
                                        MenuSelection::GenerateConfig
                                    } else {
                                        MenuSelection::Cancel
                                    }
                                }
                                MenuSelection::GenerateConfig => MenuSelection::Cancel,
                                MenuSelection::Cancel => {
                                    if !self.env_exists {
                                        MenuSelection::GenerateEnv
                                    } else if !self.config_exists {
                                        MenuSelection::GenerateConfig
                                    } else {
                                        MenuSelection::Proceed
                                    }
                                }
                            };
                        }
                        KeyCode::Enter => {
                            return Ok(Some(self.menu_selection.clone()));
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            return Ok(Some(MenuSelection::Cancel));
                        }
                        KeyCode::Char('c') => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                return Ok(Some(MenuSelection::Cancel));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(None)
    }

    fn handle_form_events(&mut self) -> Result<Option<bool>> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if self.form_data.editing {
                        match key.code {
                            KeyCode::Enter => {
                                self.form_data.editing = false;
                            }
                            KeyCode::Esc => {
                                self.form_data.editing = false;
                            }
                            KeyCode::Char(c) => {
                                self.form_data.get_current_value_mut().push(c);
                            }
                            KeyCode::Backspace => {
                                self.form_data.get_current_value_mut().pop();
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Up => {
                                if self.form_data.current_field > 0 {
                                    self.form_data.current_field -= 1;
                                }
                            }
                            KeyCode::Down | KeyCode::Tab => {
                                if self.form_data.current_field < 3 {
                                    self.form_data.current_field += 1;
                                }
                            }
                            KeyCode::Enter => {
                                self.form_data.editing = true;
                            }
                            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                if self.form_data.validate() {
                                    return Ok(Some(true));
                                }
                            }
                            KeyCode::Esc | KeyCode::Char('q') => {
                                return Ok(Some(false));
                            }
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                return Ok(Some(false));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn generate_env_file(&self) -> Result<()> {
        let project_root = Self::get_project_root();
        let env_path = project_root.join(".env");

        let env_content = format!(
            r#"COMPOSE_PROJECT_NAME=analytics
PLATFORM=linux/amd64

PROJECT_DIR=.

# service port
ANALYTICS_ENGINE_PORT=8080
ANALYTICS_ENGINE_SQL_PORT=7432
ANALYTICS_AI_SERVICE_PORT={}
ANALYTICS_UI_PORT=3000
IBIS_SERVER_PORT=8000
ANALYTICS_UI_ENDPOINT=http://analytics-ui:${{ANALYTICS_UI_PORT}}

ANALYTICS_ENGINE_ENDPOINT=http://analytics-engine:${{ANALYTICS_ENGINE_PORT}}
IBIS_SERVER_ENDPOINT=http://ibis-server:${{IBIS_SERVER_PORT}}
ANALYTICS_AI_ENDPOINT=http://analytics-service:${{ANALYTICS_AI_SERVICE_PORT}}

# ai service settings
QDRANT_HOST=qdrant
SHOULD_FORCE_DEPLOY=1

# vendor keys
OPENAI_API_KEY={}

# version
ANALYTICS_PRODUCT_VERSION=0.27.0
ANALYTICS_ENGINE_VERSION=0.18.3
ANALYTICS_AI_SERVICE_VERSION=main-ffe8ce0
IBIS_SERVER_VERSION=0.18.3
ANALYTICS_UI_VERSION=main-ffe8ce0
ANALYTICS_BOOTSTRAP_VERSION=0.1.5

# user id
USER_UUID=demo-user-{}

# other services
POSTHOG_API_KEY=
POSTHOG_HOST=https://app.posthog.com
TELEMETRY_ENABLED=false
GENERATION_MODEL={}
LANGFUSE_SECRET_KEY=
LANGFUSE_PUBLIC_KEY=

# ports
HOST_PORT={}
AI_SERVICE_FORWARD_PORT={}

# Analytics UI
EXPERIMENTAL_ENGINE_RUST_VERSION=false
DB_TYPE=pg
PG_URL=postgres://demo:demo123@northwind-db:5432/northwind
NEXT_PUBLIC_TELEMETRY_ENABLED=false

# Analytics Engine
LOCAL_STORAGE=.

# Northwind Database
POSTGRES_DB=northwind
POSTGRES_USER=demo
POSTGRES_PASSWORD=demo123

# Analytics Service
PYTHONUNBUFFERED=1
CONFIG_PATH=/app/config.yaml
"#,
            self.form_data.ai_service_port,
            self.form_data.openai_api_key,
            uuid::Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("123"),
            self.form_data.generation_model,
            self.form_data.host_port,
            self.form_data.ai_service_port,
        );

        fs::write(env_path, env_content)?;
        Ok(())
    }

    fn generate_config_yaml(&self) -> Result<()> {
        let project_root = Self::get_project_root();
        let config_path = project_root.join("config.yaml");
        let config_content = include_str!("../config_template.yaml");
        fs::write(config_path, config_content)?;
        Ok(())
    }

    async fn run_docker_compose(&mut self) -> Result<()> {
        self.add_log("🔨 Step 1/2: Building images (no cache)...");
        self.add_log("📦 Executing: docker compose build --no-cache");

        let mut build_child = Command::new("docker")
            .args(&["compose", "build", "--no-cache"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let build_stdout = build_child.stdout.take().expect("Failed to capture stdout");
        let build_stderr = build_child.stderr.take().expect("Failed to capture stderr");

        let mut build_stdout_reader = BufReader::new(build_stdout).lines();
        let mut build_stderr_reader = BufReader::new(build_stderr).lines();

        loop {
            tokio::select! {
                result = build_stdout_reader.next_line() => {
                    match result {
                        Ok(Some(line)) => self.process_log_line(&line),
                        Ok(None) => break,
                        Err(e) => {
                            self.add_log(&format!("❌ Error reading stdout: {}", e));
                            break;
                        }
                    }
                }
                result = build_stderr_reader.next_line() => {
                    match result {
                        Ok(Some(line)) => self.process_log_line(&line),
                        Ok(None) => break,
                        Err(e) => {
                            self.add_log(&format!("❌ Error reading stderr: {}", e));
                            break;
                        }
                    }
                }
            }
        }

        let build_status = build_child.wait().await?;

        if !build_status.success() {
            return Err(color_eyre::eyre::eyre!("Docker Compose build failed"));
        }

        self.add_log("✅ Build completed successfully!");
        self.progress = 50.0;

        self.add_log("🚀 Step 2/2: Starting services...");
        self.add_log("📦 Executing: docker compose up -d");

        let mut up_child = Command::new("docker")
            .args(&["compose", "up", "-d"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let up_stdout = up_child.stdout.take().expect("Failed to capture stdout");
        let up_stderr = up_child.stderr.take().expect("Failed to capture stderr");

        let mut up_stdout_reader = BufReader::new(up_stdout).lines();
        let mut up_stderr_reader = BufReader::new(up_stderr).lines();

        loop {
            tokio::select! {
                result = up_stdout_reader.next_line() => {
                    match result {
                        Ok(Some(line)) => self.process_log_line(&line),
                        Ok(None) => break,
                        Err(e) => {
                            self.add_log(&format!("❌ Error reading stdout: {}", e));
                            break;
                        }
                    }
                }
                result = up_stderr_reader.next_line() => {
                    match result {
                        Ok(Some(line)) => self.process_log_line(&line),
                        Ok(None) => break,
                        Err(e) => {
                            self.add_log(&format!("❌ Error reading stderr: {}", e));
                            break;
                        }
                    }
                }
            }
        }

        let up_status = up_child.wait().await?;

        if up_status.success() {
            self.add_log("✅ All services started successfully!");
            self.progress = 100.0;
            Ok(())
        } else {
            Err(color_eyre::eyre::eyre!("Docker Compose up failed"))
        }
    }

    fn process_log_line(&mut self, line: &str) {
        let lower = line.to_lowercase();

        if lower.contains("pulling") {
            if let Some(service) = self.extract_service_name(line) {
                self.current_service = service.clone();
                self.add_log(&format!("⬇️  Pulling image for {}...", service));
            }
        } else if lower.contains("pulled") {
            self.add_log("✓ Image pulled");
        } else if lower.contains("creating") {
            if let Some(service) = self.extract_service_name(line) {
                self.current_service = service.clone();
                self.add_log(&format!("🔨 Creating container {}...", service));
            }
        } else if lower.contains("created") {
            self.add_log("✓ Container created");
        } else if lower.contains("starting") {
            if let Some(service) = self.extract_service_name(line) {
                self.current_service = service.clone();
                self.add_log(&format!("▶️  Starting service {}...", service));
            }
        } else if lower.contains("started") {
            self.completed_services += 1;
            self.progress =
                50.0 + (self.completed_services as f64 / self.total_services as f64) * 50.0;
            self.add_log(&format!(
                "✅ Service started ({}/{})",
                self.completed_services, self.total_services
            ));
        } else if lower.contains("running") {
            self.add_log("🟢 Service is running");
        } else if lower.contains("error") || lower.contains("failed") {
            self.add_log(&format!("❌ {}", line));
        } else if !line.trim().is_empty() {
            self.add_log(&format!("ℹ️  {}", line));
        }
    }

    fn extract_service_name(&self, line: &str) -> Option<String> {
        let services = vec![
            "analytics-service",
            "qdrant",
            "northwind-db",
            "analytics-ui",
        ];

        for service in services {
            if line.to_lowercase().contains(service) {
                return Some(service.to_string());
            }
        }
        None
    }

    fn add_log(&mut self, message: &str) {
        self.logs.push(message.to_string());

        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        match &self.state {
            AppState::Confirmation => self.render_confirmation(frame),
            AppState::EnvSetup => self.render_env_setup(frame),
            AppState::Installing => self.render_installing(frame),
            AppState::Success => self.render_success(frame),
            AppState::Error(err) => self.render_error(frame, err),
        }
    }

    fn render_confirmation(&self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(10),   // Content
                Constraint::Length(5), // Menu
                Constraint::Length(2), // Help
            ])
            .split(area);

        // Title
        let title = Paragraph::new("🚀 Analytics Installer v0.1.0")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL))
            .centered();
        frame.render_widget(title, chunks[0]);

        // Content - File Status
        let all_files_exist = self.env_exists && self.config_exists;

        let mut content_lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Configuration Files:",
                Style::default().fg(if all_files_exist {
                    Color::Green
                } else {
                    Color::Yellow
                }),
            )),
            Line::from(""),
        ];

        // .env status
        content_lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                if self.env_exists { "✓" } else { "✗" },
                Style::default().fg(if self.env_exists {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
            Span::raw(" .env"),
            if !self.env_exists {
                Span::styled(" (missing)", Style::default().fg(Color::Red))
            } else {
                Span::raw("")
            },
        ]));

        // config.yaml status
        content_lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                if self.config_exists { "✓" } else { "✗" },
                Style::default().fg(if self.config_exists {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
            Span::raw(" config.yaml"),
            if !self.config_exists {
                Span::styled(" (missing)", Style::default().fg(Color::Red))
            } else {
                Span::raw("")
            },
        ]));

        content_lines.push(Line::from(""));

        if all_files_exist {
            content_lines.push(Line::from(Span::styled(
                "✅ All configuration files ready!",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )));
            content_lines.push(Line::from(""));
            content_lines.push(Line::from("Services to be started:"));
            content_lines.push(Line::from("  • analytics-service"));
            content_lines.push(Line::from("  • qdrant"));
            content_lines.push(Line::from("  • northwind-db (PostgreSQL demo)"));
            content_lines.push(Line::from("  • analytics-ui"));
        } else {
            content_lines.push(Line::from(Span::styled(
                "⚠️  Some configuration files are missing!",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            content_lines.push(Line::from(
                "Please generate the missing files before proceeding.",
            ));
        }

        let content = Paragraph::new(content_lines)
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .centered();
        frame.render_widget(content, chunks[1]);

        // Menu
        let mut menu_lines = vec![Line::from("")];

        // Show appropriate menu options
        if !self.env_exists {
            let style = if self.menu_selection == MenuSelection::GenerateEnv {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan)
            };
            menu_lines.push(Line::from(Span::styled("[ Generate .env ]", style)));
        }

        if !self.config_exists {
            let style = if self.menu_selection == MenuSelection::GenerateConfig {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan)
            };
            menu_lines.push(Line::from(Span::styled("[ Generate config.yaml ]", style)));
        }

        if all_files_exist {
            let style = if self.menu_selection == MenuSelection::Proceed {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Green)
            };
            menu_lines.push(Line::from(Span::styled(
                "[ Proceed with Installation ]",
                style,
            )));
        }

        let cancel_style = if self.menu_selection == MenuSelection::Cancel {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red)
        };
        menu_lines.push(Line::from(Span::styled("[ Cancel ]", cancel_style)));

        let menu = Paragraph::new(menu_lines)
            .block(Block::default().borders(Borders::ALL).title("Menu"))
            .centered();
        frame.render_widget(menu, chunks[2]);

        // Help
        let help = Paragraph::new("Use ↑↓ to navigate, Enter to select, Esc to cancel")
            .style(Style::default().fg(Color::DarkGray))
            .centered();
        frame.render_widget(help, chunks[3]);
    }

    fn render_env_setup(&self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(15),   // Form
                Constraint::Length(2), // Help
            ])
            .split(area);

        // Title
        let title = Paragraph::new("🔧 Generate .env File")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL))
            .centered();
        frame.render_widget(title, chunks[0]);

        // Form
        let mut form_lines = vec![
            Line::from(""),
            Line::from("Please provide the following information:"),
            Line::from(""),
        ];

        // Field 0: OpenAI API Key
        let field0_style = if self.form_data.current_field == 0 {
            if self.form_data.editing {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default().fg(Color::White)
        };

        let key_display = if self.form_data.openai_api_key.is_empty() {
            "_".repeat(40)
        } else {
            format!(
                "{}{}",
                &self.form_data.openai_api_key,
                "_".repeat(40 - self.form_data.openai_api_key.len().min(40))
            )
        };

        form_lines.push(Line::from(vec![
            Span::styled("OpenAI API Key: ", field0_style),
            Span::styled(&key_display[..40.min(key_display.len())], field0_style),
            Span::styled(" *", Style::default().fg(Color::Red)),
        ]));
        form_lines.push(Line::from(""));

        // Field 1: Generation Model
        let field1_style = if self.form_data.current_field == 1 {
            if self.form_data.editing {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default().fg(Color::White)
        };

        form_lines.push(Line::from(vec![
            Span::styled("Generation Model: ", field1_style),
            Span::styled(&self.form_data.generation_model, field1_style),
        ]));
        form_lines.push(Line::from(""));

        // Field 2: UI Port
        let field2_style = if self.form_data.current_field == 2 {
            if self.form_data.editing {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default().fg(Color::White)
        };

        form_lines.push(Line::from(vec![
            Span::styled("UI Port: ", field2_style),
            Span::styled(&self.form_data.host_port, field2_style),
        ]));
        form_lines.push(Line::from(""));

        // Field 3: AI Service Port
        let field3_style = if self.form_data.current_field == 3 {
            if self.form_data.editing {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default().fg(Color::White)
        };

        form_lines.push(Line::from(vec![
            Span::styled("AI Service Port: ", field3_style),
            Span::styled(&self.form_data.ai_service_port, field3_style),
        ]));
        form_lines.push(Line::from(""));

        if !self.form_data.error_message.is_empty() {
            form_lines.push(Line::from(""));
            form_lines.push(Line::from(Span::styled(
                &self.form_data.error_message,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )));
        }

        form_lines.push(Line::from(""));
        form_lines.push(Line::from(Span::styled(
            "* Required field",
            Style::default().fg(Color::DarkGray),
        )));

        let form = Paragraph::new(form_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Configuration Form"),
        );
        frame.render_widget(form, chunks[1]);

        // Help
        let help_text = if self.form_data.editing {
            "Type to edit, Enter to finish, Esc to cancel"
        } else {
            "↑↓ to navigate, Enter to edit, Ctrl+S to save, Esc to cancel"
        };

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .centered();
        frame.render_widget(help, chunks[2]);
    }

    fn render_installing(&self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(2),
            ])
            .split(area);

        let title = Paragraph::new("🔄 Installing Analytics... Please wait")
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL))
            .centered();
        frame.render_widget(title, chunks[0]);

        let progress_width = (chunks[1].width as f64 - 10.0) * (self.progress / 100.0);
        let filled = "█".repeat(progress_width as usize);
        let empty = "░".repeat((chunks[1].width as usize - 10 - progress_width as usize).max(0));

        let progress_text = format!("[{}{}] {:.0}%", filled, empty, self.progress);
        let progress = Paragraph::new(progress_text)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL).title("Progress"))
            .centered();
        frame.render_widget(progress, chunks[1]);

        let current = if !self.current_service.is_empty() {
            format!(
                "Current: {} ({}/{})",
                self.current_service, self.completed_services, self.total_services
            )
        } else {
            "Initializing...".to_string()
        };

        let current_widget = Paragraph::new(current)
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .centered();
        frame.render_widget(current_widget, chunks[2]);

        let log_lines: Vec<Line> = self
            .logs
            .iter()
            .map(|log| {
                let style = if log.contains("❌") || log.contains("error") {
                    Style::default().fg(Color::Red)
                } else if log.contains("✅") || log.contains("started") {
                    Style::default().fg(Color::Green)
                } else if log.contains("⬇️") {
                    Style::default().fg(Color::Blue)
                } else if log.contains("🔨") {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };

                Line::from(Span::styled(log.clone(), style))
            })
            .collect();

        let logs_widget = Paragraph::new(log_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("📋 Installation Logs"),
            )
            .wrap(Wrap { trim: false })
            .scroll((
                self.logs
                    .len()
                    .saturating_sub(chunks[3].height as usize - 2) as u16,
                0,
            ));
        frame.render_widget(logs_widget, chunks[3]);

        let help = Paragraph::new("Press Ctrl+C to cancel")
            .style(Style::default().fg(Color::DarkGray))
            .centered();
        frame.render_widget(help, chunks[4]);
    }

    fn render_success(&self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Min(10),
                Constraint::Length(2),
            ])
            .split(area);

        let title = Paragraph::new("✅ Installation Complete!")
            .style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL))
            .centered();
        frame.render_widget(title, chunks[0]);

        let message = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Analytics has been successfully installed!",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("All services are now running. You can access Analytics UI at:"),
            Line::from(Span::styled(
                "http://localhost:3000",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::UNDERLINED),
            )),
            Line::from(""),
        ];

        let message_widget = Paragraph::new(message)
            .block(Block::default().borders(Borders::ALL).title("Success"))
            .centered();
        frame.render_widget(message_widget, chunks[1]);

        let log_lines: Vec<Line> = self
            .logs
            .iter()
            .rev()
            .take(10)
            .rev()
            .map(|log| Line::from(Span::styled(log.clone(), Style::default().fg(Color::White))))
            .collect();

        let logs_widget = Paragraph::new(log_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Installation Summary"),
        );
        frame.render_widget(logs_widget, chunks[2]);

        let help = Paragraph::new("Press Ctrl+C to exit")
            .style(Style::default().fg(Color::DarkGray))
            .centered();
        frame.render_widget(help, chunks[3]);
    }

    fn render_error(&self, frame: &mut Frame, error: &str) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Min(10),
                Constraint::Length(2),
            ])
            .split(area);

        let title = Paragraph::new("❌ Installation Failed")
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL))
            .centered();
        frame.render_widget(title, chunks[0]);

        let message = vec![
            Line::from(""),
            Line::from(Span::styled(
                "An error occurred:",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(error, Style::default().fg(Color::White))),
            Line::from(""),
        ];

        let message_widget = Paragraph::new(message)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Error Details"),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(message_widget, chunks[1]);

        let log_lines: Vec<Line> = self
            .logs
            .iter()
            .map(|log| Line::from(Span::styled(log.clone(), Style::default().fg(Color::White))))
            .collect();

        let logs_widget = Paragraph::new(log_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Installation Logs"),
            )
            .wrap(Wrap { trim: false })
            .scroll((
                self.logs
                    .len()
                    .saturating_sub(chunks[2].height as usize - 2) as u16,
                0,
            ));
        frame.render_widget(logs_widget, chunks[2]);

        let help = Paragraph::new("Press Ctrl+C to exit")
            .style(Style::default().fg(Color::DarkGray))
            .centered();
        frame.render_widget(help, chunks[3]);
    }
}
