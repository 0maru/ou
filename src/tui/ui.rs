use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use super::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Worktree list
    let items: Vec<ListItem> = app
        .worktrees
        .iter()
        .enumerate()
        .map(|(i, wt)| {
            let branch = wt.branch.as_deref().unwrap_or("(detached)");
            let short_head = if wt.head.len() >= 7 {
                &wt.head[..7]
            } else {
                &wt.head
            };

            let mut spans = vec![
                Span::styled(
                    format!("{branch:<30}"),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    short_head.to_string(),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw("  "),
                Span::styled(
                    wt.path.to_string_lossy().to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
            ];

            if wt.is_locked {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    "[locked]",
                    Style::default().fg(Color::Red),
                ));
            }

            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(spans)).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Worktrees ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(list, chunks[0]);

    // Status bar
    let status_text = if let Some(ref msg) = app.status_message {
        msg.clone()
    } else {
        "q: quit  j/k: navigate  d: delete  r: refresh  Enter: open".to_string()
    };

    let status = Paragraph::new(status_text).block(
        Block::default()
            .title(" Status ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(status, chunks[1]);
}
