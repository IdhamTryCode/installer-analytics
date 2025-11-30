use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::MenuSelection;

pub struct ConfirmationView<'a> {
    pub env_exists: bool,
    pub config_exists: bool,
    pub menu_selection: &'a MenuSelection,
}

pub fn render_confirmation(frame: &mut Frame, view: &ConfirmationView<'_>) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(5),
            Constraint::Length(2),
        ])
        .split(area);

    let title = Paragraph::new("🚀 Analytics Installer v0.1.0")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL))
        .centered();
    frame.render_widget(title, chunks[0]);

    let all_files_exist = view.env_exists && view.config_exists;

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

    content_lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            if view.env_exists { "✓" } else { "✗" },
            Style::default().fg(if view.env_exists {
                Color::Green
            } else {
                Color::Red
            }),
        ),
        Span::raw(" .env"),
        if !view.env_exists {
            Span::styled(" (missing)", Style::default().fg(Color::Red))
        } else {
            Span::raw("")
        },
    ]));

    content_lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            if view.config_exists { "✓" } else { "✗" },
            Style::default().fg(if view.config_exists {
                Color::Green
            } else {
                Color::Red
            }),
        ),
        Span::raw(" config.yaml"),
        if !view.config_exists {
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

    let mut menu_lines = vec![Line::from("")];

    if !view.env_exists {
        let style = if *view.menu_selection == MenuSelection::GenerateEnv {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan)
        };
        menu_lines.push(Line::from(Span::styled("[ Generate .env ]", style)));
    }

    if !view.config_exists {
        let style = if *view.menu_selection == MenuSelection::GenerateConfig {
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
        let style = if *view.menu_selection == MenuSelection::Proceed {
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

    let cancel_style = if *view.menu_selection == MenuSelection::Cancel {
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

    let help = Paragraph::new("Use ↑↓ to navigate, Enter to select, Esc to cancel")
        .style(Style::default().fg(Color::DarkGray))
        .centered();
    frame.render_widget(help, chunks[3]);
}
