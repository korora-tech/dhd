use crate::tui::app::{AppState, TuiApp};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

pub fn draw(frame: &mut Frame, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(frame.area());

    draw_header(frame, chunks[0]);

    match app.state {
        AppState::ModuleSelection => draw_module_selection(frame, chunks[1], app),
        AppState::PlanView => draw_plan_view(frame, chunks[1], app),
        AppState::Applying => draw_applying_view(frame, chunks[1], app),
    }

    draw_footer(frame, chunks[2], app);
}

fn draw_header(frame: &mut Frame, area: Rect) {
    let header = Paragraph::new("DHD - Declarative Home Deployments")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let help_text = match app.state {
        AppState::ModuleSelection => {
            "↑/↓: Navigate | Space: Select | Enter: Plan | Ctrl+A: Select All | Ctrl+D: Deselect | q: Quit"
        }
        AppState::PlanView => "↑/↓: Scroll | Enter/a: Apply | Esc: Back | q: Back",
        AppState::Applying => "Applying configuration...",
    };

    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(footer, area);
}

fn draw_module_selection(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Filter
            Constraint::Min(0),    // Module list
        ])
        .split(area);

    // Filter input
    let filter_block = Block::default().title("Filter").borders(Borders::ALL);
    let filter_text = Paragraph::new(app.filter.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(filter_block);
    frame.render_widget(filter_text, chunks[0]);

    // Module list
    let modules = app.filtered_modules();
    let items: Vec<ListItem> = modules
        .iter()
        .enumerate()
        .map(|(i, module)| {
            let selected = app.selected_modules.contains(&module.name);
            let is_current = i == app.current_selection;

            let mut spans = vec![
                Span::raw(if selected { "[x] " } else { "[ ] " }),
                Span::styled(
                    &module.name,
                    Style::default().fg(if is_current {
                        Color::Yellow
                    } else {
                        Color::White
                    }),
                ),
            ];

            if let Some(desc) = &module.description {
                spans.push(Span::raw(" - "));
                spans.push(Span::styled(desc, Style::default().fg(Color::DarkGray)));
            }

            if !module.dependencies.is_empty() {
                spans.push(Span::raw(" (deps: "));
                spans.push(Span::styled(
                    module.dependencies.join(", "),
                    Style::default().fg(Color::Blue),
                ));
                spans.push(Span::raw(")"));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!("Modules ({} selected)", app.selected_modules.len()))
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(app.current_selection));
    frame.render_stateful_widget(list, chunks[1], &mut list_state);
}

fn draw_plan_view(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let plan_text = if let Some(plan) = &app.plan {
        plan.iter()
            .enumerate()
            .map(|(i, action)| format!("{}. {}", i + 1, action))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        "No plan generated".to_string()
    };

    let paragraph = Paragraph::new(plan_text)
        .block(
            Block::default()
                .title("Execution Plan")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true })
        .scroll((app.scroll_offset as u16, 0));

    frame.render_widget(paragraph, area);
}

fn draw_applying_view(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let status = app.apply_status.lock().unwrap();

    let mut text = vec![];

    if status.is_running {
        text.push(Line::from(Span::styled(
            "⏳ Applying configuration...",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        text.push(Line::from(""));
        text.push(Line::from(format!("Current: {}", status.current_action)));

        if status.total_actions > 0 {
            text.push(Line::from(format!(
                "Progress: {}/{} actions",
                status.completed_actions, status.total_actions
            )));
        }
    } else if let Some(error) = &status.error {
        text.push(Line::from(Span::styled(
            "❌ Apply failed!",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            error,
            Style::default().fg(Color::Red),
        )));
        text.push(Line::from(""));
        text.push(Line::from("Press ESC or q to go back"));
    } else if status.completed_actions > 0 {
        text.push(Line::from(Span::styled(
            "✅ Apply completed successfully!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )));
        text.push(Line::from(""));
        text.push(Line::from(format!(
            "Completed {} actions",
            status.completed_actions
        )));
        text.push(Line::from(""));
        text.push(Line::from("Press ESC or q to go back"));
    } else {
        text.push(Line::from(status.current_action.clone()));
    }

    let paragraph = Paragraph::new(text)
        .block(Block::default().title("Applying").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}
