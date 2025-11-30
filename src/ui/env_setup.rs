use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::FormData;

pub struct EnvSetupView<'a> {
    pub form_data: &'a FormData,
}

pub fn render_env_setup(frame: &mut Frame, view: &EnvSetupView<'_>) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(15),
            Constraint::Length(2),
        ])
        .split(area);

    let title = Paragraph::new("🔧 Generate .env File")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL))
        .centered();
    frame.render_widget(title, chunks[0]);

    let data = view.form_data;

    let mut form_lines = vec![
        Line::from(""),
        Line::from("Please provide the following information:"),
        Line::from(""),
    ];

    let field0_style = if data.current_field == 0 {
        if data.editing {
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

    let key_display = if data.openai_api_key.is_empty() {
        "_".repeat(40)
    } else {
        format!(
            "{}{}",
            &data.openai_api_key,
            "_".repeat(40 - data.openai_api_key.len().min(40))
        )
    };

    form_lines.push(Line::from(vec![
        Span::styled("OpenAI API Key: ", field0_style),
        Span::styled(&key_display[..40.min(key_display.len())], field0_style),
        Span::styled(" *", Style::default().fg(Color::Red)),
    ]));
    form_lines.push(Line::from(""));

    let field1_style = if data.current_field == 1 {
        if data.editing {
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
        Span::styled(&data.generation_model, field1_style),
    ]));
    form_lines.push(Line::from(""));

    let field2_style = if data.current_field == 2 {
        if data.editing {
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
        Span::styled(&data.host_port, field2_style),
    ]));
    form_lines.push(Line::from(""));

    let field3_style = if data.current_field == 3 {
        if data.editing {
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
        Span::styled(&data.ai_service_port, field3_style),
    ]));
    form_lines.push(Line::from(""));

    if !data.error_message.is_empty() {
        form_lines.push(Line::from(""));
        form_lines.push(Line::from(Span::styled(
            &data.error_message,
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

    let help_text = if data.editing {
        "Type to edit, Enter to finish, Esc to cancel"
    } else {
        "↑↓ to navigate, Enter to edit, Ctrl+S to save, Esc to cancel"
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .centered();
    frame.render_widget(help, chunks[2]);
}
