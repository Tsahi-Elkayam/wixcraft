//! Interactive TUI for wix-easy
//!
//! Provides a terminal-based wizard for creating installer configurations.

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use std::io;
use std::path::PathBuf;

use wix_easy::{
    Architecture, EnvAction, EnvScope, EnvironmentDef, FileDef, InstallDef, InstallScope,
    InstallerDef, PackageDef, RegistryDef, RegistryValue, ServiceDef, ServiceStart, ShortcutDef,
    ShortcutLocation, UiDef, UiStyle, UpgradeDef,
};

/// Current screen/step in the wizard
#[derive(Debug, Clone, Copy, PartialEq)]
enum Screen {
    Welcome,
    Package,
    InstallDir,
    Files,
    Shortcuts,
    Registry,
    Services,
    Environment,
    UI,
    Review,
    Complete,
}

/// Which field is currently being edited
#[derive(Debug, Clone, Copy, PartialEq)]
enum EditingField {
    None,
    // Package fields
    Name,
    Version,
    Manufacturer,
    Description,
    Scope,
    // Install fields
    Directory,
    // File fields
    FilePath,
    // Shortcut fields
    ShortcutName,
    ShortcutTarget,
    ShortcutLocation,
    // Registry fields
    RegistryKey,
    RegistryValueName,
    RegistryValue,
    // Service fields
    ServiceName,
    ServiceExe,
    ServiceDisplayName,
    // Environment fields
    EnvName,
    EnvValue,
    // UI
    UIStyle,
}

/// Application state
pub struct App {
    screen: Screen,
    editing: EditingField,
    input_buffer: String,
    cursor_pos: usize,

    // Status message
    status: String,

    // The installer definition being built
    def: InstallerDef,

    // Temporary state for adding items
    temp_file: FileDef,
    temp_shortcut: ShortcutDef,
    temp_registry: RegistryDef,
    temp_service: ServiceDef,
    temp_env: EnvironmentDef,

    // List selection
    list_index: usize,

    // Output path
    output_path: Option<PathBuf>,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Welcome,
            editing: EditingField::None,
            input_buffer: String::new(),
            cursor_pos: 0,
            status: String::new(),
            def: InstallerDef {
                package: PackageDef {
                    name: String::new(),
                    version: "1.0.0".to_string(),
                    manufacturer: String::new(),
                    description: String::new(),
                    product_code: None,
                    upgrade_code: None,
                    icon: None,
                    license: None,
                    scope: InstallScope::PerMachine,
                    architecture: Architecture::X64,
                },
                install: InstallDef {
                    directory: "ProgramFiles/MyCompany/MyApp".to_string(),
                    files: Vec::new(),
                    directories: Vec::new(),
                },
                features: Vec::new(),
                shortcuts: Vec::new(),
                registry: Vec::new(),
                environment: Vec::new(),
                services: Vec::new(),
                prerequisites: Vec::new(),
                ui: UiDef::default(),
                upgrade: UpgradeDef::default(),
            },
            temp_file: FileDef {
                src: String::new(),
                dest: String::new(),
                vital: true,
                key_path: false,
            },
            temp_shortcut: ShortcutDef {
                name: String::new(),
                target: String::new(),
                location: ShortcutLocation::StartMenu,
                working_dir: None,
                arguments: None,
                icon: None,
                description: None,
            },
            temp_registry: RegistryDef {
                key: String::new(),
                values: std::collections::HashMap::new(),
            },
            temp_service: ServiceDef {
                name: String::new(),
                display_name: None,
                executable: String::new(),
                description: None,
                start: ServiceStart::Auto,
                arguments: None,
                account: None,
            },
            temp_env: EnvironmentDef {
                name: String::new(),
                value: String::new(),
                action: EnvAction::Set,
                scope: EnvScope::User,
            },
            list_index: 0,
            output_path: None,
        }
    }

    fn next_screen(&mut self) {
        self.screen = match self.screen {
            Screen::Welcome => Screen::Package,
            Screen::Package => Screen::InstallDir,
            Screen::InstallDir => Screen::Files,
            Screen::Files => Screen::Shortcuts,
            Screen::Shortcuts => Screen::Registry,
            Screen::Registry => Screen::Services,
            Screen::Services => Screen::Environment,
            Screen::Environment => Screen::UI,
            Screen::UI => Screen::Review,
            Screen::Review => Screen::Complete,
            Screen::Complete => Screen::Complete,
        };
        self.editing = EditingField::None;
        self.status.clear();
    }

    fn prev_screen(&mut self) {
        self.screen = match self.screen {
            Screen::Welcome => Screen::Welcome,
            Screen::Package => Screen::Welcome,
            Screen::InstallDir => Screen::Package,
            Screen::Files => Screen::InstallDir,
            Screen::Shortcuts => Screen::Files,
            Screen::Registry => Screen::Shortcuts,
            Screen::Services => Screen::Registry,
            Screen::Environment => Screen::Services,
            Screen::UI => Screen::Environment,
            Screen::Review => Screen::UI,
            Screen::Complete => Screen::Review,
        };
        self.editing = EditingField::None;
        self.status.clear();
    }

    fn start_editing(&mut self, field: EditingField) {
        self.editing = field;
        self.input_buffer = match field {
            EditingField::Name => self.def.package.name.clone(),
            EditingField::Version => self.def.package.version.clone(),
            EditingField::Manufacturer => self.def.package.manufacturer.clone(),
            EditingField::Description => self.def.package.description.clone(),
            EditingField::Directory => self.def.install.directory.clone(),
            EditingField::FilePath => self.temp_file.src.clone(),
            EditingField::ShortcutName => self.temp_shortcut.name.clone(),
            EditingField::ShortcutTarget => self.temp_shortcut.target.clone(),
            EditingField::ServiceName => self.temp_service.name.clone(),
            EditingField::ServiceExe => self.temp_service.executable.clone(),
            EditingField::ServiceDisplayName => self.temp_service.display_name.clone().unwrap_or_default(),
            EditingField::EnvName => self.temp_env.name.clone(),
            EditingField::EnvValue => self.temp_env.value.clone(),
            EditingField::RegistryKey => self.temp_registry.key.clone(),
            _ => String::new(),
        };
        self.cursor_pos = self.input_buffer.len();
    }

    fn finish_editing(&mut self) {
        match self.editing {
            EditingField::Name => self.def.package.name = self.input_buffer.clone(),
            EditingField::Version => self.def.package.version = self.input_buffer.clone(),
            EditingField::Manufacturer => self.def.package.manufacturer = self.input_buffer.clone(),
            EditingField::Description => self.def.package.description = self.input_buffer.clone(),
            EditingField::Directory => self.def.install.directory = self.input_buffer.clone(),
            EditingField::FilePath => self.temp_file.src = self.input_buffer.clone(),
            EditingField::ShortcutName => self.temp_shortcut.name = self.input_buffer.clone(),
            EditingField::ShortcutTarget => self.temp_shortcut.target = self.input_buffer.clone(),
            EditingField::ServiceName => self.temp_service.name = self.input_buffer.clone(),
            EditingField::ServiceExe => self.temp_service.executable = self.input_buffer.clone(),
            EditingField::ServiceDisplayName => {
                if self.input_buffer.is_empty() {
                    self.temp_service.display_name = None;
                } else {
                    self.temp_service.display_name = Some(self.input_buffer.clone());
                }
            }
            EditingField::EnvName => self.temp_env.name = self.input_buffer.clone(),
            EditingField::EnvValue => self.temp_env.value = self.input_buffer.clone(),
            EditingField::RegistryKey => self.temp_registry.key = self.input_buffer.clone(),
            _ => {}
        }
        self.editing = EditingField::None;
        self.input_buffer.clear();
    }

    fn cancel_editing(&mut self) {
        self.editing = EditingField::None;
        self.input_buffer.clear();
    }

    fn handle_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) => {
                self.input_buffer.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.input_buffer.remove(self.cursor_pos);
                }
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.input_buffer.len() {
                    self.input_buffer.remove(self.cursor_pos);
                }
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_pos < self.input_buffer.len() {
                    self.cursor_pos += 1;
                }
            }
            KeyCode::Home => self.cursor_pos = 0,
            KeyCode::End => self.cursor_pos = self.input_buffer.len(),
            _ => {}
        }
    }

    fn add_file(&mut self) {
        if !self.temp_file.src.is_empty() {
            self.def.install.files.push(self.temp_file.clone());
            self.temp_file = FileDef {
                src: String::new(),
                dest: String::new(),
                vital: true,
                key_path: false,
            };
            self.status = "File added".to_string();
        }
    }

    fn add_shortcut(&mut self) {
        if !self.temp_shortcut.name.is_empty() && !self.temp_shortcut.target.is_empty() {
            self.def.shortcuts.push(self.temp_shortcut.clone());
            self.temp_shortcut = ShortcutDef {
                name: String::new(),
                target: String::new(),
                location: ShortcutLocation::StartMenu,
                working_dir: None,
                arguments: None,
                icon: None,
                description: None,
            };
            self.status = "Shortcut added".to_string();
        }
    }

    fn add_service(&mut self) {
        if !self.temp_service.name.is_empty() && !self.temp_service.executable.is_empty() {
            self.def.services.push(self.temp_service.clone());
            self.temp_service = ServiceDef {
                name: String::new(),
                display_name: None,
                executable: String::new(),
                description: None,
                start: ServiceStart::Auto,
                arguments: None,
                account: None,
            };
            self.status = "Service added".to_string();
        }
    }

    fn add_environment(&mut self) {
        if !self.temp_env.name.is_empty() {
            self.def.environment.push(self.temp_env.clone());
            self.temp_env = EnvironmentDef {
                name: String::new(),
                value: String::new(),
                action: EnvAction::Set,
                scope: EnvScope::User,
            };
            self.status = "Environment variable added".to_string();
        }
    }

    fn cycle_scope(&mut self) {
        self.def.package.scope = match self.def.package.scope {
            InstallScope::PerMachine => InstallScope::PerUser,
            InstallScope::PerUser => InstallScope::PerMachine,
        };
    }

    fn cycle_shortcut_location(&mut self) {
        self.temp_shortcut.location = match self.temp_shortcut.location {
            ShortcutLocation::StartMenu => ShortcutLocation::Desktop,
            ShortcutLocation::Desktop => ShortcutLocation::Both,
            ShortcutLocation::Both => ShortcutLocation::StartMenu,
        };
    }

    fn cycle_ui_style(&mut self) {
        self.def.ui.style = match self.def.ui.style {
            UiStyle::Minimal => UiStyle::Basic,
            UiStyle::Basic => UiStyle::Full,
            UiStyle::Full => UiStyle::None,
            UiStyle::None => UiStyle::Minimal,
        };
    }

    fn cycle_service_start(&mut self) {
        self.temp_service.start = match self.temp_service.start {
            ServiceStart::Auto => ServiceStart::Manual,
            ServiceStart::Manual => ServiceStart::Disabled,
            ServiceStart::Disabled => ServiceStart::Auto,
            _ => ServiceStart::Auto,
        };
    }

    fn cycle_env_action(&mut self) {
        self.temp_env.action = match self.temp_env.action {
            EnvAction::Set => EnvAction::Append,
            EnvAction::Append => EnvAction::Prepend,
            EnvAction::Prepend => EnvAction::Set,
        };
    }

    fn cycle_env_scope(&mut self) {
        self.temp_env.scope = match self.temp_env.scope {
            EnvScope::User => EnvScope::System,
            EnvScope::System => EnvScope::User,
        };
    }

    fn generate_yaml(&self) -> String {
        serde_yaml::to_string(&self.def).unwrap_or_else(|e| format!("Error: {}", e))
    }

    fn generate_wix(&self) -> Result<String, wix_easy::EasyError> {
        self.def.generate_wix(None)
    }
}

/// Run the interactive TUI
pub fn run_interactive(output: Option<PathBuf>) -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();
    app.output_path = output;

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Handle result
    if let Err(e) = result {
        println!("Error: {}", e);
        return Err(e);
    }

    // Output the configuration if we completed
    if app.screen == Screen::Complete {
        if let Some(ref path) = app.output_path {
            let yaml = app.generate_yaml();
            std::fs::write(path, yaml)?;
            println!("Configuration saved to: {}", path.display());
        } else {
            println!("\n--- Generated YAML Configuration ---\n");
            println!("{}", app.generate_yaml());
        }
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            // Handle editing mode
            if app.editing != EditingField::None {
                match key.code {
                    KeyCode::Enter => app.finish_editing(),
                    KeyCode::Esc => app.cancel_editing(),
                    _ => app.handle_input(key.code),
                }
                continue;
            }

            // Handle screen-specific input
            match app.screen {
                Screen::Welcome => match key.code {
                    KeyCode::Enter | KeyCode::Char('n') => app.next_screen(),
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => {}
                },
                Screen::Package => match key.code {
                    KeyCode::Char('1') | KeyCode::Char('n') => app.start_editing(EditingField::Name),
                    KeyCode::Char('2') | KeyCode::Char('v') => app.start_editing(EditingField::Version),
                    KeyCode::Char('3') | KeyCode::Char('m') => app.start_editing(EditingField::Manufacturer),
                    KeyCode::Char('4') | KeyCode::Char('d') => app.start_editing(EditingField::Description),
                    KeyCode::Char('5') | KeyCode::Char('s') => app.cycle_scope(),
                    KeyCode::Enter | KeyCode::Tab => {
                        if !app.def.package.name.is_empty() && !app.def.package.manufacturer.is_empty() {
                            app.next_screen();
                        } else {
                            app.status = "Name and Manufacturer are required".to_string();
                        }
                    }
                    KeyCode::Backspace => app.prev_screen(),
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => {}
                },
                Screen::InstallDir => match key.code {
                    KeyCode::Char('e') => app.start_editing(EditingField::Directory),
                    KeyCode::Enter | KeyCode::Tab => app.next_screen(),
                    KeyCode::Backspace => app.prev_screen(),
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => {}
                },
                Screen::Files => match key.code {
                    KeyCode::Char('a') => app.start_editing(EditingField::FilePath),
                    KeyCode::Char('d') => {
                        if !app.def.install.files.is_empty() && app.list_index < app.def.install.files.len() {
                            app.def.install.files.remove(app.list_index);
                            if app.list_index >= app.def.install.files.len() && app.list_index > 0 {
                                app.list_index -= 1;
                            }
                        }
                    }
                    KeyCode::Up => {
                        if app.list_index > 0 {
                            app.list_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if app.list_index < app.def.install.files.len().saturating_sub(1) {
                            app.list_index += 1;
                        }
                    }
                    KeyCode::Enter => {
                        if app.editing == EditingField::FilePath {
                            app.finish_editing();
                            app.add_file();
                        } else {
                            app.add_file();
                        }
                    }
                    KeyCode::Tab => app.next_screen(),
                    KeyCode::Backspace => app.prev_screen(),
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => {}
                },
                Screen::Shortcuts => match key.code {
                    KeyCode::Char('n') => app.start_editing(EditingField::ShortcutName),
                    KeyCode::Char('t') => app.start_editing(EditingField::ShortcutTarget),
                    KeyCode::Char('l') => app.cycle_shortcut_location(),
                    KeyCode::Char('a') => app.add_shortcut(),
                    KeyCode::Char('d') => {
                        if !app.def.shortcuts.is_empty() && app.list_index < app.def.shortcuts.len() {
                            app.def.shortcuts.remove(app.list_index);
                        }
                    }
                    KeyCode::Up => {
                        if app.list_index > 0 {
                            app.list_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if app.list_index < app.def.shortcuts.len().saturating_sub(1) {
                            app.list_index += 1;
                        }
                    }
                    KeyCode::Enter | KeyCode::Tab => app.next_screen(),
                    KeyCode::Backspace => app.prev_screen(),
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => {}
                },
                Screen::Registry => match key.code {
                    KeyCode::Enter | KeyCode::Tab => app.next_screen(),
                    KeyCode::Backspace => app.prev_screen(),
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => {}
                },
                Screen::Services => match key.code {
                    KeyCode::Char('n') => app.start_editing(EditingField::ServiceName),
                    KeyCode::Char('e') => app.start_editing(EditingField::ServiceExe),
                    KeyCode::Char('d') => app.start_editing(EditingField::ServiceDisplayName),
                    KeyCode::Char('s') => app.cycle_service_start(),
                    KeyCode::Char('a') => app.add_service(),
                    KeyCode::Enter | KeyCode::Tab => app.next_screen(),
                    KeyCode::Backspace => app.prev_screen(),
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => {}
                },
                Screen::Environment => match key.code {
                    KeyCode::Char('n') => app.start_editing(EditingField::EnvName),
                    KeyCode::Char('v') => app.start_editing(EditingField::EnvValue),
                    KeyCode::Char('t') => app.cycle_env_action(),
                    KeyCode::Char('s') => app.cycle_env_scope(),
                    KeyCode::Char('a') => app.add_environment(),
                    KeyCode::Enter | KeyCode::Tab => app.next_screen(),
                    KeyCode::Backspace => app.prev_screen(),
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => {}
                },
                Screen::UI => match key.code {
                    KeyCode::Char('s') => app.cycle_ui_style(),
                    KeyCode::Enter | KeyCode::Tab => app.next_screen(),
                    KeyCode::Backspace => app.prev_screen(),
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => {}
                },
                Screen::Review => match key.code {
                    KeyCode::Char('g') => {
                        app.next_screen();
                    }
                    KeyCode::Backspace => app.prev_screen(),
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => {}
                },
                Screen::Complete => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter => return Ok(()),
                    _ => {}
                },
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Status/Help
        ])
        .split(f.area());

    // Title bar
    let title = match app.screen {
        Screen::Welcome => " wix-easy Interactive Mode ",
        Screen::Package => " Package Information ",
        Screen::InstallDir => " Installation Directory ",
        Screen::Files => " Files to Install ",
        Screen::Shortcuts => " Shortcuts ",
        Screen::Registry => " Registry Settings ",
        Screen::Services => " Windows Services ",
        Screen::Environment => " Environment Variables ",
        Screen::UI => " User Interface ",
        Screen::Review => " Review Configuration ",
        Screen::Complete => " Complete ",
    };

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    let title_para = Paragraph::new(title)
        .block(title_block)
        .alignment(Alignment::Center)
        .style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(title_para, chunks[0]);

    // Main content
    let content = render_screen(app);
    let content_block = Block::default().borders(Borders::ALL);
    let content_para = Paragraph::new(content)
        .block(content_block)
        .wrap(Wrap { trim: false });
    f.render_widget(content_para, chunks[1]);

    // Status/help bar
    let help_text = render_help(app);
    let help_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::DarkGray));
    let help_para = Paragraph::new(help_text).block(help_block);
    f.render_widget(help_para, chunks[2]);

    // Render input popup if editing
    if app.editing != EditingField::None {
        render_input_popup(f, app);
    }
}

fn render_screen(app: &App) -> String {
    match app.screen {
        Screen::Welcome => {
            format!(
                r#"
  Welcome to wix-easy Interactive Mode!

  This wizard will help you create a Windows installer configuration.
  You'll be guided through:

    - Package information (name, version, manufacturer)
    - Installation directory
    - Files to install
    - Shortcuts (desktop, start menu)
    - Registry settings
    - Windows services
    - Environment variables
    - User interface options

  At the end, you'll get a YAML configuration file that can be used
  to generate WiX XML and build an MSI installer.

  Press ENTER to begin or Q to quit.
"#
            )
        }
        Screen::Package => {
            let scope_str = match app.def.package.scope {
                InstallScope::PerMachine => "per-machine (all users)",
                InstallScope::PerUser => "per-user (current user only)",
            };
            format!(
                r#"
  Configure your package information:

  [1] Name:         {}
  [2] Version:      {}
  [3] Manufacturer: {}
  [4] Description:  {}
  [5] Scope:        {}

  Press 1-5 to edit a field, or TAB/ENTER to continue.
  {}
"#,
                if app.def.package.name.is_empty() { "<required>" } else { &app.def.package.name },
                app.def.package.version,
                if app.def.package.manufacturer.is_empty() { "<required>" } else { &app.def.package.manufacturer },
                if app.def.package.description.is_empty() { "<optional>" } else { &app.def.package.description },
                scope_str,
                if !app.status.is_empty() { format!("\n  {}", app.status) } else { String::new() }
            )
        }
        Screen::InstallDir => {
            format!(
                r#"
  Configure the installation directory:

  Directory: {}

  Standard paths:
    - ProgramFiles/Company/App  (recommended)
    - ProgramFiles64/Company/App
    - LocalAppData/Company/App

  Press E to edit, or TAB/ENTER to continue.
"#,
                app.def.install.directory
            )
        }
        Screen::Files => {
            let mut files_list = String::new();
            for (i, file) in app.def.install.files.iter().enumerate() {
                let marker = if i == app.list_index { ">" } else { " " };
                files_list.push_str(&format!("  {} {}\n", marker, file.src));
            }
            if files_list.is_empty() {
                files_list = "  (no files added yet)\n".to_string();
            }
            format!(
                r#"
  Add files to install (supports glob patterns like ./bin/*):

{}
  New file: {}

  Press A to add file path, D to delete selected, TAB to continue.
  {}
"#,
                files_list,
                if app.temp_file.src.is_empty() { "<enter path>" } else { &app.temp_file.src },
                if !app.status.is_empty() { format!("\n  {}", app.status) } else { String::new() }
            )
        }
        Screen::Shortcuts => {
            let loc_str = match app.temp_shortcut.location {
                ShortcutLocation::StartMenu => "startmenu",
                ShortcutLocation::Desktop => "desktop",
                ShortcutLocation::Both => "both",
            };
            let mut list = String::new();
            for (i, s) in app.def.shortcuts.iter().enumerate() {
                let marker = if i == app.list_index { ">" } else { " " };
                list.push_str(&format!("  {} {} -> {}\n", marker, s.name, s.target));
            }
            if list.is_empty() {
                list = "  (no shortcuts added yet)\n".to_string();
            }
            format!(
                r#"
  Add shortcuts:

{}
  New shortcut:
    [N] Name:     {}
    [T] Target:   {}
    [L] Location: {}

  Press A to add, D to delete selected, TAB to continue.
  {}
"#,
                list,
                if app.temp_shortcut.name.is_empty() { "<enter name>" } else { &app.temp_shortcut.name },
                if app.temp_shortcut.target.is_empty() { "<enter target>" } else { &app.temp_shortcut.target },
                loc_str,
                if !app.status.is_empty() { format!("\n  {}", app.status) } else { String::new() }
            )
        }
        Screen::Registry => {
            let mut list = String::new();
            for reg in &app.def.registry {
                list.push_str(&format!("  {}\n", reg.key));
            }
            if list.is_empty() {
                list = "  (no registry entries - skip for now)\n".to_string();
            }
            format!(
                r#"
  Registry settings (advanced):

{}
  Press TAB to continue or BACKSPACE to go back.
"#,
                list
            )
        }
        Screen::Services => {
            let start_str = match app.temp_service.start {
                ServiceStart::Auto => "auto",
                ServiceStart::Manual => "manual",
                ServiceStart::Disabled => "disabled",
                _ => "auto",
            };
            let mut list = String::new();
            for s in &app.def.services {
                list.push_str(&format!("  {} ({})\n", s.name, s.executable));
            }
            if list.is_empty() {
                list = "  (no services added yet)\n".to_string();
            }
            format!(
                r#"
  Windows Services:

{}
  New service:
    [N] Name:         {}
    [E] Executable:   {}
    [D] Display Name: {}
    [S] Start Type:   {}

  Press A to add, TAB to continue.
  {}
"#,
                list,
                if app.temp_service.name.is_empty() { "<enter name>" } else { &app.temp_service.name },
                if app.temp_service.executable.is_empty() { "<enter exe>" } else { &app.temp_service.executable },
                app.temp_service.display_name.as_deref().unwrap_or("<same as name>"),
                start_str,
                if !app.status.is_empty() { format!("\n  {}", app.status) } else { String::new() }
            )
        }
        Screen::Environment => {
            let action_str = match app.temp_env.action {
                EnvAction::Set => "set",
                EnvAction::Append => "append",
                EnvAction::Prepend => "prepend",
            };
            let scope_str = match app.temp_env.scope {
                EnvScope::User => "user",
                EnvScope::System => "system",
            };
            let mut list = String::new();
            for e in &app.def.environment {
                list.push_str(&format!("  {}={}\n", e.name, e.value));
            }
            if list.is_empty() {
                list = "  (no environment variables added yet)\n".to_string();
            }
            format!(
                r#"
  Environment Variables:

{}
  New variable:
    [N] Name:   {}
    [V] Value:  {}
    [T] Type:   {}
    [S] Scope:  {}

  Press A to add, TAB to continue.
  {}
"#,
                list,
                if app.temp_env.name.is_empty() { "<enter name>" } else { &app.temp_env.name },
                if app.temp_env.value.is_empty() { "<enter value>" } else { &app.temp_env.value },
                action_str,
                scope_str,
                if !app.status.is_empty() { format!("\n  {}", app.status) } else { String::new() }
            )
        }
        Screen::UI => {
            let style_str = match app.def.ui.style {
                UiStyle::Minimal => "minimal (simple progress bar)",
                UiStyle::Basic => "basic (install directory selection)",
                UiStyle::Full => "full (feature tree selection)",
                UiStyle::None => "none (silent install only)",
            };
            format!(
                r#"
  User Interface Options:

  [S] UI Style: {}

  Styles:
    - minimal: Simple progress bar, no user choices
    - basic:   User can choose install directory
    - full:    User can select features to install
    - none:    Silent install only, no UI

  Press S to cycle through options, TAB to continue.
"#,
                style_str
            )
        }
        Screen::Review => {
            format!(
                r#"
  Review your configuration:

  Package:
    Name:         {}
    Version:      {}
    Manufacturer: {}
    Scope:        {:?}

  Install:
    Directory:    {}
    Files:        {} files
    Shortcuts:    {} shortcuts
    Services:     {} services
    Environment:  {} variables

  UI Style:       {:?}

  Press G to generate and save, BACKSPACE to go back.
"#,
                app.def.package.name,
                app.def.package.version,
                app.def.package.manufacturer,
                app.def.package.scope,
                app.def.install.directory,
                app.def.install.files.len(),
                app.def.shortcuts.len(),
                app.def.services.len(),
                app.def.environment.len(),
                app.def.ui.style
            )
        }
        Screen::Complete => {
            format!(
                r#"
  Configuration complete!

  Your installer configuration has been created.
  {}

  Next steps:
    1. Edit the YAML file if needed
    2. Run: wix-easy generate config.yaml -o installer.wxs
    3. Build with: wix build installer.wxs

  Press ENTER or Q to exit.
"#,
                if let Some(ref path) = app.output_path {
                    format!("Saved to: {}", path.display())
                } else {
                    "YAML will be printed to stdout.".to_string()
                }
            )
        }
    }
}

fn render_help(app: &App) -> String {
    if app.editing != EditingField::None {
        return "ENTER: Save | ESC: Cancel | Arrow keys: Navigate".to_string();
    }

    match app.screen {
        Screen::Welcome => "ENTER: Start | Q: Quit".to_string(),
        Screen::Package => "1-5: Edit field | TAB: Next | BACKSPACE: Back | Q: Quit".to_string(),
        Screen::InstallDir => "E: Edit | TAB: Next | BACKSPACE: Back | Q: Quit".to_string(),
        Screen::Files => "A: Add | D: Delete | Up/Down: Select | TAB: Next | Q: Quit".to_string(),
        Screen::Shortcuts => "N/T/L: Edit fields | A: Add | D: Delete | TAB: Next | Q: Quit".to_string(),
        Screen::Registry => "TAB: Next | BACKSPACE: Back | Q: Quit".to_string(),
        Screen::Services => "N/E/D/S: Edit fields | A: Add | TAB: Next | Q: Quit".to_string(),
        Screen::Environment => "N/V/T/S: Edit fields | A: Add | TAB: Next | Q: Quit".to_string(),
        Screen::UI => "S: Cycle style | TAB: Next | BACKSPACE: Back | Q: Quit".to_string(),
        Screen::Review => "G: Generate | BACKSPACE: Back | Q: Quit".to_string(),
        Screen::Complete => "ENTER/Q: Exit".to_string(),
    }
}

fn render_input_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 20, f.area());

    // Clear the area first
    f.render_widget(Clear, area);

    let field_name = match app.editing {
        EditingField::Name => "Product Name",
        EditingField::Version => "Version",
        EditingField::Manufacturer => "Manufacturer",
        EditingField::Description => "Description",
        EditingField::Directory => "Install Directory",
        EditingField::FilePath => "File Path",
        EditingField::ShortcutName => "Shortcut Name",
        EditingField::ShortcutTarget => "Shortcut Target",
        EditingField::ServiceName => "Service Name",
        EditingField::ServiceExe => "Service Executable",
        EditingField::ServiceDisplayName => "Display Name",
        EditingField::EnvName => "Variable Name",
        EditingField::EnvValue => "Variable Value",
        EditingField::RegistryKey => "Registry Key",
        _ => "Edit",
    };

    let block = Block::default()
        .title(format!(" {} ", field_name))
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Input text with cursor
    let display_text = format!("{}|", &app.input_buffer[..app.cursor_pos]);
    let display_text = format!("{}{}", display_text, &app.input_buffer[app.cursor_pos..]);

    let input = Paragraph::new(display_text)
        .style(Style::default().fg(Color::White));
    f.render_widget(input, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
