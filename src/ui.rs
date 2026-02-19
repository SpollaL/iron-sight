use crate::app::App;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::{Cell, Paragraph, Row, Table};
use ratatui::Frame;

pub fn ui(frame: &mut Frame, app: &mut App) {
    let header_cells = Row::new(app.headers.iter().map(|header| Cell::from(header.as_str())));
    let rows = app
        .records
        .iter()
        .map(|record| Row::new(record.iter().map(|field| Cell::from(field.as_str()))))
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
    let bar = format!(
        " Row {}/{} | Col {}/{} | {} ",
        app.state.selected().map_or(0, |i| i + 1),
        app.records.len(),
        app.state.selected_column().map_or(0, |i| i + 1),
        app.headers.len(),
        app.filepath
    );
    let bar = Paragraph::new(bar).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_stateful_widget(table, chunks[0], &mut app.state);
    frame.render_widget(bar, chunks[1]);
}
