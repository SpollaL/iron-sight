use crate::app::{App, Mode};
use crate::ui::ui;
use crossterm::event;
use polars::prelude::DataType;

const PAGE_SCROLL_AMOUNT: u16 = 20;

pub fn run_app(
    terminal: &mut ratatui::DefaultTerminal,
    mut app: App,
) -> Result<(), Box<dyn std::error::Error>> {
    while !app.should_quit {
        terminal.draw(|frame| ui(frame, &mut app))?;

        if let event::Event::Key(key) = event::read()? {
            match app.mode {
                Mode::Normal => match key.code {
                    event::KeyCode::Char('q') => app.should_quit = true,
                    event::KeyCode::Down => app.state.select_next(),
                    event::KeyCode::Up => app.state.select_previous(),
                    event::KeyCode::Left => app.state.select_previous_column(),
                    event::KeyCode::Right => app.state.select_next_column(),
                    event::KeyCode::Char('j') => app.state.select_next(),
                    event::KeyCode::Char('k') => app.state.select_previous(),
                    event::KeyCode::Char('h') => app.state.select_previous_column(),
                    event::KeyCode::Char('l') => app.state.select_next_column(),
                    event::KeyCode::Char('g') => app.state.select_first(),
                    event::KeyCode::Char('G') => app.state.select_last(),
                    event::KeyCode::PageDown => app.state.scroll_down_by(PAGE_SCROLL_AMOUNT),
                    event::KeyCode::PageUp => app.state.scroll_up_by(PAGE_SCROLL_AMOUNT),
                    event::KeyCode::Home => app.state.select_first(),
                    event::KeyCode::End => app.state.select_last(),
                    event::KeyCode::Char('_') => autofit_column(&mut app),
                    event::KeyCode::Char('/') => enter_search_mode(&mut app),
                    event::KeyCode::Char('n') => go_to_next_search_result(&mut app),
                    event::KeyCode::Char('N') => go_to_previous_search_result(&mut app),
                    event::KeyCode::Char('f') => enter_filter_mode(&mut app),
                    event::KeyCode::Char('F') => clear_filters(&mut app),
                    event::KeyCode::Char('s') => app.sort_by_column(),
                    event::KeyCode::Char('S') => app.show_stats = !app.show_stats,
                    _ => {}
                },
                Mode::Search => match key.code {
                    event::KeyCode::Backspace => pop_char_from_search_query(&mut app),
                    event::KeyCode::Enter => to_first_search_query_result(&mut app),
                    event::KeyCode::Char(c) => push_char_to_search_query(&mut app, c),
                    event::KeyCode::Esc => from_search_to_normal_mode(&mut app),
                    _ => {}
                },
                Mode::Filter => match key.code {
                    event::KeyCode::Backspace => pop_char_from_filter_query(&mut app),
                    event::KeyCode::Enter => to_normal_mode_with_filter(&mut app),
                    event::KeyCode::Char(c) => push_char_to_filter_query(&mut app, c),
                    event::KeyCode::Esc => from_filter_to_normal_mode(&mut app),
                    _ => {}
                },
            }
        }
    }
    Ok(())
}

fn autofit_column(app: &mut App) {
    if let Some(col_idx) = app.state.selected_column() {
        let header_width = app.headers.get(col_idx).map_or(0, |h| h.len()) as u16;
        let col_name = app.headers[col_idx].clone();
        let max_data = app
            .view
            .column(&col_name)
            .ok()
            .map(|col| {
                let str_series = col
                    .as_series()
                    .unwrap()
                    .cast(&DataType::String)
                    .unwrap();
                str_series
                    .str()
                    .unwrap()
                    .into_iter()
                    .map(|v| v.map_or(0, |s| s.len()))
                    .max()
                    .unwrap_or(0) as u16
            })
            .unwrap_or(0);
        app.column_widths[col_idx] = max_data.max(header_width);
    }
}

fn enter_search_mode(app: &mut App) {
    app.mode = Mode::Search;
    app.search_results = Vec::new();
    app.search_query = String::new();
}

fn enter_filter_mode(app: &mut App) {
    app.mode = Mode::Filter;
    app.filter_input = String::new();
}

fn push_char_to_search_query(app: &mut App, c: char) {
    app.search_query.push(c);
    app.update_search();
}

fn push_char_to_filter_query(app: &mut App, c: char) {
    app.filter_input.push(c);
    app.update_filter();
}

fn pop_char_from_search_query(app: &mut App) {
    app.search_query.pop();
    app.update_search();
}

fn pop_char_from_filter_query(app: &mut App) {
    app.filter_input.pop();
    app.update_filter();
}

fn to_first_search_query_result(app: &mut App) {
    if app.search_results.is_empty() {
        return;
    }
    app.state
        .select(Some(app.search_results[app.search_cursor]));
    app.mode = Mode::Normal;
}

fn to_normal_mode_with_filter(app: &mut App) {
    app.mode = Mode::Normal;
    if !app.filter_input.is_empty() {
        app.filters.push((
            app.state.selected_column().unwrap_or(0),
            app.filter_input.clone(),
        ));
        app.filter_input = String::new();
        app.update_filter();
    }
}

fn from_search_to_normal_mode(app: &mut App) {
    app.mode = Mode::Normal;
    app.search_results = Vec::new();
    app.search_query = String::new();
    app.search_cursor = 0;
}

fn from_filter_to_normal_mode(app: &mut App) {
    app.mode = Mode::Normal;
    app.filter_input = String::new();
}

fn clear_filters(app: &mut App) {
    app.filter_input = String::new();
    app.filters = Vec::new();
    app.update_filter();
}

fn go_to_next_search_result(app: &mut App) {
    if app.search_results.is_empty() {
        return;
    }
    app.search_cursor = if app.search_cursor < app.search_results.len() - 1 {
        app.search_cursor + 1
    } else {
        0
    };
    app.state
        .select(Some(app.search_results[app.search_cursor]));
}

fn go_to_previous_search_result(app: &mut App) {
    if app.search_results.is_empty() {
        return;
    }
    app.search_cursor = if app.search_cursor > 0 {
        app.search_cursor - 1
    } else {
        app.search_results.len() - 1
    };
    app.state
        .select(Some(app.search_results[app.search_cursor]));
}
