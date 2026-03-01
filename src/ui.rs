use crate::app::{AggFunc, App, Mode};
use catppuccin::PALETTE;
use polars::prelude::DataType;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table};
use ratatui::Frame;

fn c(color: catppuccin::Color) -> Color {
    Color::Rgb(color.rgb.r, color.rgb.g, color.rgb.b)
}

pub fn ui(frame: &mut Frame, app: &mut App) {
    let m = &PALETTE.mocha.colors;

    let header_cells = Row::new((0..app.headers.len()).map(|i| {
        Cell::from(app.header_label(i)).style(
            Style::default()
                .fg(c(m.lavender))
                .add_modifier(Modifier::BOLD),
        )
    }))
    .style(Style::default().bg(c(m.surface0)));

    let str_columns: Vec<_> = app
        .view
        .get_columns()
        .iter()
        .map(|col| col.as_series().unwrap().cast(&DataType::String).unwrap())
        .collect();

    let rows: Vec<Row> = (0..app.view.height())
        .map(|i| {
            let bg = if i % 2 == 0 { c(m.base) } else { c(m.mantle) };
            Row::new(
                str_columns
                    .iter()
                    .map(|s| Cell::from(s.str().unwrap().get(i).unwrap_or("").to_string()))
                    .collect::<Vec<Cell>>(),
            )
            .style(Style::default().bg(bg).fg(c(m.text)))
        })
        .collect();

    let widths: Vec<Constraint> = app
        .column_widths
        .iter()
        .map(|w| Constraint::Length(*w))
        .collect();

    let table = Table::new(rows, widths)
        .header(header_cells.bottom_margin(1))
        .block(
            Block::default()
                .title(format!(" {} ", app.file_path))
                .title_style(
                    Style::default()
                        .fg(c(m.blue))
                        .add_modifier(Modifier::BOLD),
                )
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(c(m.overlay0)))
                .style(Style::default().bg(c(m.base))),
        )
        .row_highlight_style(Style::default().bg(c(m.surface0)))
        .column_highlight_style(Style::default().bg(c(m.surface1)))
        .cell_highlight_style(
            Style::default()
                .bg(c(m.blue))
                .fg(c(m.base))
                .add_modifier(Modifier::BOLD),
        );

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let (bar_text, bar_style) = get_bar(app, m);
    let bar = Paragraph::new(bar_text).style(bar_style);

    frame.render_stateful_widget(table, chunks[0], &mut app.state);
    frame.render_widget(bar, chunks[1]);

    if app.show_stats {
        let col = app.state.selected_column().unwrap_or(0);
        let stats = app.compute_stats(col);
        let area = centered_rect(40, 40, frame.area());
        frame.render_widget(Clear, area);
        let content = format!(
            "\n Count:  {}\n Min:    {}\n Max:    {}\n Mean:   {}\n Median: {}",
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
                Block::default()
                    .title(" Column Stats ")
                    .title_style(
                        Style::default()
                            .fg(c(m.mauve))
                            .add_modifier(Modifier::BOLD),
                    )
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(c(m.mauve))),
            )
            .style(Style::default().bg(c(m.surface0)).fg(c(m.text)));
        frame.render_widget(popup, area);
    }

    if app.show_help {
        let area = centered_rect(55, 80, frame.area());
        frame.render_widget(Clear, area);
        let popup = Paragraph::new(help_text(m))
            .block(
                Block::default()
                    .title(" Help — press ? or Esc to close ")
                    .title_style(
                        Style::default()
                            .fg(c(m.lavender))
                            .add_modifier(Modifier::BOLD),
                    )
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(c(m.lavender))),
            )
            .style(Style::default().bg(c(m.surface0)).fg(c(m.text)));
        frame.render_widget(popup, area);
    }
}

fn get_bar(app: &App, m: &catppuccin::FlavorColors) -> (String, Style) {
    match app.mode {
        Mode::Search => (
            format!(" /{}_ ", app.search_query),
            Style::default()
                .bg(c(m.yellow))
                .fg(c(m.base))
                .add_modifier(Modifier::BOLD),
        ),
        Mode::Filter => (
            format!(" f {}_ ", app.filter_input),
            Style::default()
                .bg(c(m.sapphire))
                .fg(c(m.base))
                .add_modifier(Modifier::BOLD),
        ),
        Mode::Normal => {
            let (text, fg) = if app.groupby_active {
                let key_names = app
                    .saved_headers
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
                (
                    format!(
                        " ◆ GROUPED  By: {} | Agg: {} | {} rows ",
                        key_names,
                        agg_summary,
                        app.view.height()
                    ),
                    c(m.yellow),
                )
            } else if !app.groupby_keys.is_empty() {
                let key_names = app
                    .groupby_keys
                    .iter()
                    .map(|&i| app.headers[i].as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                (
                    format!(" GroupBy: {} | press B to execute ", key_names),
                    c(m.peach),
                )
            } else if !app.search_results.is_empty() {
                (
                    format!(
                        " [{}/{}]  {} ",
                        app.search_cursor + 1,
                        app.search_results.len(),
                        app.search_query
                    ),
                    c(m.sky),
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
                (
                    format!(
                        " {} | Row {}/{} | Col {}/{} | {} ",
                        filter_summary,
                        app.state.selected().map_or(0, |i| i + 1),
                        app.view.height(),
                        app.state.selected_column().map_or(0, |i| i + 1),
                        app.headers.len(),
                        app.file_path
                    ),
                    c(m.teal),
                )
            } else {
                (
                    format!(
                        " Row {}/{} | Col {}/{} | {}  ? help ",
                        app.state.selected().map_or(0, |i| i + 1),
                        app.view.height(),
                        app.state.selected_column().map_or(0, |i| i + 1),
                        app.headers.len(),
                        app.file_path
                    ),
                    c(m.subtext1),
                )
            };
            (text, Style::default().bg(c(m.surface0)).fg(fg))
        }
    }
}

fn help_text(m: &catppuccin::FlavorColors) -> Text<'static> {
    let section = |title: &'static str| {
        Line::from(vec![
            Span::raw(" "),
            Span::styled(
                title,
                Style::default()
                    .fg(c(m.lavender))
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    };
    let key = |k: &'static str, desc: &'static str| {
        Line::from(vec![
            Span::styled(format!("  {:<14}", k), Style::default().fg(c(m.blue))),
            Span::styled(desc, Style::default().fg(c(m.text))),
        ])
    };
    Text::from(vec![
        Line::raw(""),
        section("Navigation"),
        key("j / ↓", "Move down"),
        key("k / ↑", "Move up"),
        key("h / ←", "Move left"),
        key("l / →", "Move right"),
        key("g / Home", "First row"),
        key("G / End", "Last row"),
        key("PageDown", "Scroll down 20 rows"),
        key("PageUp", "Scroll up 20 rows"),
        Line::raw(""),
        section("Search"),
        key("/", "Enter search mode"),
        key("Enter", "Jump to first match"),
        key("n / N", "Next / previous match"),
        key("Esc", "Exit search"),
        Line::raw(""),
        section("Filter"),
        key("f", "Enter filter mode (current column)"),
        key("Enter", "Apply filter"),
        key("F", "Clear all filters"),
        key("Esc", "Discard input"),
        Line::raw(""),
        section("Sort"),
        key("s", "Sort by column (toggles asc / desc)"),
        Line::raw(""),
        section("Group By"),
        key("b", "Toggle group-by key [K]"),
        key("a", "Cycle aggregation  [Σ μ # ↓ ↑]"),
        key("B", "Execute / clear group-by"),
        Line::raw(""),
        section("Other"),
        key("_", "Autofit column width"),
        key("S", "Toggle column stats popup"),
        key("?", "Toggle this help"),
        key("q", "Quit"),
        Line::raw(""),
    ])
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
