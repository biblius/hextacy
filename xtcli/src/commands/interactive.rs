pub mod layout;
pub mod proxy;
pub mod render;

use self::proxy::{CommandInfo, CommandProxy, OptionField, OptionsProxy, SubcommandProxy};
use crate::commands::interactive::proxy::CryptoProxy;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{fmt::Debug, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

use super::crypto::{PWOpts, SecretOpts};

#[derive(Debug)]
pub struct AlxApp {
    // Command
    commands: StatefulList<CommandProxy>,
    active_cmd: Option<CommandProxy>,

    // Subcommand
    subcommands: StatefulList<SubcommandProxy>,
    active_sub: Option<SubcommandProxy>,

    // Options
    options: StatefulList<OptionField>,
    active_opts: Option<OptionsProxy>,
    selected_opt_field: Option<OptionField>,

    /// Flag indicating that a subcommand was freshly selected, used for rendering
    sub_changed: bool,
    options_changed: bool,

    /// The currently active window (box) in the terminal
    active_window: Window,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Window {
    Command,
    Subcommand,
    Options,
}

impl AlxApp {
    fn new() -> Self {
        use CommandProxy::*;
        let commands = vec![Crypto, Envex];
        let mut this = Self {
            commands: StatefulList::with_items(commands.clone()),
            subcommands: StatefulList::with_items(vec![]),
            options: StatefulList::with_items(vec![]),
            active_cmd: None,
            active_sub: None,
            active_opts: None,
            selected_opt_field: None,
            active_window: Window::Command,
            sub_changed: false,
            options_changed: false,
        };
        this.commands.state.select(Some(0));
        this
    }

    fn handle_up(&mut self) {
        match self.active_window {
            Window::Command => self.commands.next(),
            Window::Subcommand => self.subcommands.next(),
            Window::Options => self.options.next(),
        }
    }

    fn handle_down(&mut self) {
        match self.active_window {
            Window::Command => self.commands.previous(),
            Window::Subcommand => self.subcommands.previous(),
            Window::Options => self.options.previous(),
        }
    }

    fn handle_char(&mut self, ch: char, opt: OptionField) {
        let Window::Options = self.active_window else {
            return;
        };
        let Some(ref mut active) = self.active_opts else {
            return;
        };

        let push_or_insert = |opt: &mut Option<String>, ch: char| match opt {
            Some(ref mut val) => val.push(ch),
            None => {
                let _ = opt.insert(String::from(ch));
            }
        };

        match active {
            OptionsProxy::CrySecret(opts) => match opt {
                OptionField::Name => opts.name.push(ch),
                OptionField::Length => {
                    if ch.is_digit(10) {
                        let mut s = opts.length.to_string();
                        s.push(ch);
                        let len = s.parse::<u32>().unwrap();
                        if len > u16::MAX as u32 {
                            s.pop();
                            opts.length = s.parse().unwrap()
                        } else {
                            opts.length = len as u16;
                        }
                    }
                }
                OptionField::Encoding => push_or_insert(&mut opts.encoding, ch),
                _ => unreachable!(),
            },
            OptionsProxy::CryPW(ref mut opts) => match opt {
                OptionField::Name => opts.name.push(ch),
                OptionField::Length => {
                    if ch.is_digit(10) {
                        let mut s = opts.length.to_string();
                        s.push(ch);
                        let len = s.parse::<u16>().unwrap();
                        if len > u8::MAX as u16 {
                            s.pop();
                            opts.length = s.parse().unwrap()
                        } else {
                            opts.length = len as u8;
                        }
                    }
                }
                _ => unreachable!(),
            },
        }
    }

    fn handle_backspace(&mut self, opt: OptionField) {
        let Window::Options = self.active_window else {
            return;
        };
        let Some(ref mut active) = self.active_opts else {
            return;
        };

        let pop_opt = |opt: &mut Option<String>| {
            if let Some(ref mut val) = opt {
                val.pop();
                if val.is_empty() {
                    *opt = None;
                }
            }
        };

        match active {
            OptionsProxy::CrySecret(opts) => match opt {
                OptionField::Name => {
                    opts.name.pop();
                }
                OptionField::Length => {
                    let mut s = opts.length.to_string();
                    s.pop();
                    opts.length = s.parse().unwrap_or_default();
                }
                OptionField::Encoding => pop_opt(&mut opts.encoding),
                _ => unreachable!(),
            },
            OptionsProxy::CryPW(ref mut opts) => match opt {
                OptionField::Name => {
                    opts.name.pop();
                }
                OptionField::Length => {
                    let mut s = opts.length.to_string();
                    s.pop();
                    opts.length = s.parse().unwrap_or_default();
                }
                _ => unreachable!(),
            },
        }
    }

    fn handle_enter(&mut self) {
        match self.active_window {
            Window::Command => {
                self.active_cmd =
                    Some(self.commands.items[self.commands.state.selected().unwrap()]);
                self.active_window = Window::Subcommand;
                self.sub_changed = true;
            }
            Window::Subcommand => {
                let active = self.subcommands.items[self.subcommands.state.selected().unwrap()];
                match active {
                    SubcommandProxy::Crypto(proxy) => match proxy {
                        CryptoProxy::Secret => {
                            self.active_opts = Some(OptionsProxy::CrySecret(SecretOpts::default()))
                        }
                        CryptoProxy::Rsa => todo!(),
                        CryptoProxy::Password => {
                            self.active_opts = Some(OptionsProxy::CryPW(PWOpts::default()))
                        }
                    },
                }
                self.active_sub = Some(active);
                self.active_window = Window::Options;
                self.options_changed = true;
            }
            Window::Options => {}
        }
    }

    fn handle_esc(&mut self) -> bool {
        match self.active_window {
            Window::Command => return true,
            Window::Subcommand => {
                self.active_cmd = None;
                self.active_sub = None;
                self.subcommands.items = vec![];
                self.active_window = Window::Command;
            }
            Window::Options => {
                self.active_sub = None;
                self.active_opts = None;
                self.active_window = Window::Subcommand;
            }
        };
        false
    }

    fn update_subcommand_list(&mut self, command: CommandProxy) {
        use SubcommandProxy::*;
        self.subcommands.items = vec![];
        match command {
            CommandProxy::Crypto => {
                use CryptoProxy::*;
                self.subcommands.items = vec![Crypto(Secret), Crypto(Rsa), Crypto(Password)];
            }
            CommandProxy::Envex => {}
        }
        if self.sub_changed {
            self.subcommands.state.select(Some(0));
        }
        self.sub_changed = false;
    }

    fn update_option_list(&mut self, command: SubcommandProxy) {
        use OptionField::*;
        use SubcommandProxy::*;
        match command {
            Crypto(proxy) => match proxy {
                CryptoProxy::Secret => self.options.items = vec![Name, Length, Encoding],
                CryptoProxy::Rsa => todo!(),
                CryptoProxy::Password => self.options.items = vec![Name, Length],
            },
        }
        if self.options_changed {
            self.options.state.select(Some(0));
        }
        self.selected_opt_field = Some(self.options.items[self.options.state.selected().unwrap()]);
        self.options_changed = false;
    }
}

pub fn init_interactive() -> io::Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = AlxApp::new();
    let res = run(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

#[inline]
fn footer_cmd(app: &AlxApp) -> String {
    format!(
        "{} {} {}",
        app.commands
            .items
            .get(app.commands.state.selected().unwrap())
            .map_or_else(String::new, |cmd| cmd.command_repr()),
        app.subcommands
            .items
            .get(app.subcommands.state.selected().unwrap_or_default())
            .map_or_else(String::new, |sub| sub.command_repr()),
        app.active_opts
            .as_ref()
            .map_or_else(String::new, |opt| opt.command_repr()),
    )
}

fn run<B: Backend>(terminal: &mut Terminal<B>, mut app: AlxApp) -> io::Result<()> {
    loop {
        // Check active command and render based on it

        if let Some(active) = app.active_sub.clone() {
            terminal.draw(|f| {
                command_list(f, &mut app);
                let command = app.active_cmd.unwrap();
                subcommand_list(f, &mut app, command);
                options_list(f, &mut app, active);
                render::header(f);
                render::footer(f, &footer_cmd(&app));
            })?;
        } else if let Some(active) = app.active_cmd {
            terminal.draw(|f| {
                command_list(f, &mut app);
                subcommand_list(f, &mut app, active);
                render::header(f);
                render::footer(f, &footer_cmd(&app));
            })?;
        } else {
            terminal.draw(|f| {
                command_list(f, &mut app);
                render::header(f);
                render::footer(f, &footer_cmd(&app));
            })?;
        }

        // Check for input

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => {
                    if app.handle_esc() {
                        return Ok(());
                    }
                }
                KeyCode::Up => app.handle_up(),
                KeyCode::Down => app.handle_down(),
                KeyCode::Enter => app.handle_enter(),
                KeyCode::Backspace if app.active_window == Window::Options => {
                    app.handle_backspace(app.selected_opt_field.unwrap())
                }
                KeyCode::Char(c) if app.active_window == Window::Options => {
                    app.handle_char(c, app.selected_opt_field.unwrap())
                }
                _ => {}
            }
        }
    }
}

fn options_list<B: Backend>(frame: &mut Frame<B>, app: &mut AlxApp, subcommand: SubcommandProxy) {
    let vertical = layout::vertical(frame);
    let right = layout::horizontal(vertical[1])[2];

    app.update_option_list(subcommand);

    let options = app
        .options
        .items
        .iter()
        .map(|opt| {
            let block_width = right.width;

            create_opt_list_item(app, opt, block_width)
        })
        .collect::<Vec<ListItem>>();

    let block = Block::default()
        .title(subcommand.title())
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Rgb(30, 20, 255)));

    let options = List::new(options)
        .block(block)
        .highlight_style(Style::default().bg(Color::Rgb(40, 100, 50)));

    frame.render_stateful_widget(options, right, &mut app.options.state);
}

fn create_opt_list_item<'a>(app: &AlxApp, opt: &OptionField, width: u16) -> ListItem<'a> {
    let Some(ref active) = app.active_opts else {
        panic!("`create_opt_list` called without options")
    };

    let header = Spans::from(vec![
        Span::styled(
            format!("{:<9}", opt),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ]);

    let value = match active {
        OptionsProxy::CrySecret(opts) => match opt {
            OptionField::Name => opts.name.clone(),
            OptionField::Encoding => opts.encoding.clone().unwrap_or_default(),
            OptionField::Length => format!("{}", opts.length),
            _ => unreachable!(),
        },
        OptionsProxy::CryPW(opts) => match opt {
            OptionField::Name => opts.name.clone(),
            OptionField::Length => format!("{}", opts.length),
            _ => unreachable!(),
        },
    };

    ListItem::new(vec![
        Spans::from("-".repeat(width as usize)),
        header,
        Spans::from(""),
        value.into(),
        Spans::from(""),
    ])
}

fn subcommand_list<B: Backend>(frame: &mut Frame<B>, app: &mut AlxApp, command: CommandProxy) {
    let vertical = layout::vertical(frame);
    let middle = layout::horizontal(vertical[1])[1];

    app.update_subcommand_list(command);

    let subcommands: Vec<ListItem> = app
        .subcommands
        .items
        .iter()
        .map(|cmd| {
            let header = Spans::from(vec![
                Span::styled(
                    format!("{:<9}", cmd.title()),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
            ]);

            let selected_style = if app.active_sub.is_some() && *cmd == app.active_sub.unwrap() {
                Spans::from(vec![Span::styled(
                    format!("{}>", cmd.description(),),
                    Style::default().add_modifier(Modifier::ITALIC),
                )])
            } else {
                Spans::from(Span::styled(
                    format!("{}", cmd.description()),
                    Style::default().add_modifier(Modifier::ITALIC),
                ))
            };

            ListItem::new(vec![
                Spans::from("-".repeat(middle.width as usize)),
                header,
                Spans::from(""),
                selected_style,
                Spans::from(""),
            ])
        })
        .collect();

    let mut block = Block::default()
        .borders(Borders::ALL)
        .title(app.active_cmd.unwrap().title())
        .style(Style::default().bg(Color::Rgb(30, 20, 50)));

    if app.active_window == Window::Subcommand {
        block = block.style(Style::default().bg(Color::Rgb(50, 30, 90)))
    }
    let subcommands = List::new(subcommands)
        .block(block)
        .highlight_style(Style::default().bg(Color::Rgb(40, 100, 50)));

    frame.render_stateful_widget(subcommands, middle, &mut app.subcommands.state);
}

fn command_list<B: Backend>(frame: &mut Frame<B>, app: &mut AlxApp) {
    let vertical = layout::vertical(frame);
    let left = layout::horizontal(vertical[1])[0];

    let commands: Vec<ListItem> = app
        .commands
        .items
        .iter()
        .map(|cmd| {
            let header = Spans::from(vec![
                Span::styled(
                    format!("{:<9}", cmd.title()),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
            ]);

            let block_width = left.width;
            let selected_style = if app.active_cmd.is_some() && *cmd == app.active_cmd.unwrap() {
                Spans::from(vec![Span::styled(
                    format!(
                        "{}{}>",
                        cmd.description(),
                        "=".repeat(block_width as usize - 3 - cmd.description().len()),
                    ),
                    Style::default().add_modifier(Modifier::ITALIC),
                )])
            } else {
                Spans::from(Span::styled(
                    format!("{}", cmd.description()),
                    Style::default().add_modifier(Modifier::ITALIC),
                ))
            };

            ListItem::new(vec![
                Spans::from("-".repeat(left.width as usize)),
                header,
                Spans::from(""),
                selected_style,
                Spans::from(""),
            ])
        })
        .collect();

    let mut block = Block::default()
        .borders(Borders::ALL)
        .title("Command")
        .style(Style::default().bg(Color::Rgb(30, 20, 50)));

    if app.active_window == Window::Command {
        block = block.style(Style::default().bg(Color::Rgb(50, 30, 90)));
    }

    let commands = List::new(commands)
        .block(block)
        .highlight_style(Style::default().bg(Color::Rgb(40, 100, 50)));

    frame.render_stateful_widget(commands, left, &mut app.commands.state);
}

#[derive(Debug)]
struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
