use crate::app::{App, Mode, SortDirection};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::{Cell, Clear, Paragraph, Row, Table};
use ratatui::Frame;

pub fn ui(frame: &mut Frame, app: &mut App) {
    let header_cells = Row::new(app.headers.iter().enumerate().map(|(i, header)| {
        if app.sort_column == Some(i) {
            let dir = match app.sort_direction {
                SortDirection::Ascending => '▲',
                SortDirection::Descending => '▼',
            };
            Cell::from(format!("{} {}", header, dir))
        } else {
            Cell::from(header.as_str())
        }
    }));
    let rows = app
        .records
        .iter()
        .enumerate()
        .filter(|(i, _)| app.filter_indices.is_empty() || app.filter_indices.contains(i))
        .map(|(_, record)| Row::new(record.iter().map(|field| Cell::from(field.as_str()))))
        .collect::<Vec<Row>>();
    let widths: Vec<Constraint> = app
        .column_widths
        .iter()
        .map(|w| Constraint::Length(*w))
        .collect();
    let table = Table::new(rows, widths)
        .header(header_cells.bold().bottom_margin(1))
        .block(
            ratatui::widgets::Block::default()
                .title("CSV Viewer")
                .borders(ratatui::widgets::Borders::ALL),
        )
        .row_highlight_style(Style::default().bg(Color::DarkGray))
        .column_highlight_style(Style::default().bg(Color::DarkGray))
        .cell_highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());
    let bar = get_bar(app);
    let bar = Paragraph::new(bar).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_stateful_widget(table, chunks[0], &mut app.state);
    frame.render_widget(bar, chunks[1]);
    if app.show_stats {
        let col = app.state.selected_column().unwrap_or(0);
        let stats = app.compute_stats(col);
        let area = centered_rect(40, 40, frame.area());
        frame.render_widget(Clear, area);
        let content = format!(
            "Count: {}\nMin:   {}\nMax:   {}\nMean:  {}\nMedian: {}",
            stats.count,
            stats.min,
            stats.max,
            stats
                .mean
                .map_or("N/A".to_string(), |v| format!("{:.2}", v)),
            stats
                .median
                .map_or("N/A".to_string(), |v| format!("{:.2}", v)),
        );
        let popup = Paragraph::new(content)
            .block(
                ratatui::widgets::Block::default()
                    .title(" Column Stats ")
                    .borders(ratatui::widgets::Borders::ALL),
            )
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));
        frame.render_widget(popup, area);
    }
}

fn get_bar(app: &App) -> String {
    match app.mode {
        Mode::Normal => {
            if !app.search_results.is_empty() {
                format!(
                    " [{}]/[{}] {} ",
                    app.search_cursor + 1,
                    app.search_results.len(),
                    app.search_query
                )
            } else if !app.filter_indices.is_empty() {
                let filter_summary = app.filters.iter().map(|(col, q)| {
                    format!("[{}: {}]", app.headers.get(*col).map_or("?", |h| h), q)
                }).collect::<Vec<_>>().join(" ");
                format!(
                    " {} Row {}/{} | Col {}/{} | {} ",
                    filter_summary,
                    app.state.selected().map_or(0, |i| i + 1),
                    app.filter_indices.len(),
                    app.state.selected_column().map_or(0, |i| i + 1),
                    app.headers.len(),
                    app.file_path
                )
            } else {
                format!(
                    " Row {}/{} | Col {}/{} | {} ",
                    app.state.selected().map_or(0, |i| i + 1),
                    app.records.len(),
                    app.state.selected_column().map_or(0, |i| i + 1),
                    app.headers.len(),
                    app.file_path
                )
            }
        }
        Mode::Search => {
            format!("/{}_", app.search_query,)
        }
        Mode::Filter => {
            format!("f {}_", app.filter_input)
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}
