use crate::app::{AggFunc, App, Mode};
use polars::prelude::DataType;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::{Cell, Clear, Paragraph, Row, Table};
use ratatui::Frame;

pub fn ui(frame: &mut Frame, app: &mut App) {
    let header_cells =
        Row::new((0..app.headers.len()).map(|i| Cell::from(app.header_label(i))));
    let str_columns: Vec<_> = app
        .view
        .get_columns()
        .iter()
        .map(|col| col.as_series().unwrap().cast(&DataType::String).unwrap())
        .collect();
    let rows: Vec<Row> = (0..app.view.height())
        .map(|i| {
            Row::new(
                str_columns
                    .iter()
                    .map(|s| Cell::from(s.str().unwrap().get(i).unwrap_or("").to_string()))
                    .collect::<Vec<Cell>>(),
            )
        })
        .collect();
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
            if app.groupby_active {
                let key_names = app
                    .saved_headers // original names, since headers is now the result
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| app.groupby_keys.contains(i))
                    .map(|(_, h)| h.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                let agg_summary = app
                    .groupby_aggs
                    .iter()
                    .map(|(i, func)| {
                        let sym = match func {
                            AggFunc::Sum => "Σ",
                            AggFunc::Mean => "μ",
                            AggFunc::Count => "#",
                            AggFunc::Min => "↓",
                            AggFunc::Max => "↑",
                        };
                        format!("{}[{}]", app.saved_headers[*i], sym)
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                format!(
                    " [GROUPED] By: {} | Agg: {} | {} rows ",
                    key_names,
                    agg_summary,
                    app.view.height()
                )
            } else if !app.groupby_keys.is_empty() {
                // setup in progress but not yet executed
                let key_names = app
                    .groupby_keys
                    .iter()
                    .map(|&i| app.headers[i].as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(" GroupBy: {} | press B to execute ", key_names)
            } else if !app.search_results.is_empty() {
                format!(
                    " [{}]/[{}] {} ",
                    app.search_cursor + 1,
                    app.search_results.len(),
                    app.search_query
                )
            } else if !app.filters.is_empty() {
                let filter_summary = app
                    .filters
                    .iter()
                    .map(|(col, q)| {
                        format!("[{}: {}]", app.headers.get(*col).map_or("?", |h| h), q)
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                format!(
                    " {} Row {}/{} | Col {}/{} | {} ",
                    filter_summary,
                    app.state.selected().map_or(0, |i| i + 1),
                    app.view.height(),
                    app.state.selected_column().map_or(0, |i| i + 1),
                    app.headers.len(),
                    app.file_path
                )
            } else {
                format!(
                    " Row {}/{} | Col {}/{} | {} ",
                    app.state.selected().map_or(0, |i| i + 1),
                    app.view.height(),
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
