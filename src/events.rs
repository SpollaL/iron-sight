use crate::app::App;
use crate::ui::ui;
use crossterm::event;

pub fn run_app(
    temrinal: &mut ratatui::DefaultTerminal,
    mut app: App,
) -> Result<(), Box<dyn std::error::Error>> {
    while !app.should_quit {
        temrinal.draw(|frame| ui(frame, &mut app))?;

        if let event::Event::Key(key) = event::read()? {
            match key.code {
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
                event::KeyCode::PageDown => app.state.scroll_down_by(20),
                event::KeyCode::PageUp => app.state.scroll_up_by(20),
                event::KeyCode::Home => app.state.select_first(),
                event::KeyCode::End => app.state.select_last(),
                event::KeyCode::Char('_') => autofit_column(&mut app),
                _ => {}
            }
        }
    }
    Ok(())
}

fn autofit_column(app: &mut App) {
    if let Some(col) = app.state.selected_column() {
        let header_width = app.headers.get(col).map_or(0, |h| h.len()) as u16;
        let max_data = app
            .records
            .iter()
            .map(|r| r.get(col).map_or(0, |f| f.len()))
            .max()
            .unwrap_or(0) as u16;
        app.column_widths[col] = max_data.max(header_width);
    }
}
