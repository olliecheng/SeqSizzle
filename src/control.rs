use crate::app::{App, SearchPattern, UIMode};
use crate::{Event, Tui};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::{Color, Rect};
use std::str::FromStr;

pub enum Update {
    ToggleSearchPanelFocus(bool),
    SearchPanelInput(KeyEvent),
    EditSearchPattern(SearchPatternEdit),
    CycleSearchPattern(bool),
    ToggleUIMode,
    ScrollViewer(isize),
    WindowResize(Rect),
    Msg(String),
    Quit,
    None,
}

pub enum SearchPatternEdit {
    Delete(usize, bool), // (index, pop into edit boxes?)
    Append(SearchPattern),
}


pub fn handle_input(app: &App, tui: &Tui, input: Event) -> Update {
    match input {
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => Update::Quit,
        Event::Key(KeyEvent {
            code: KeyCode::Char('/'),
            modifiers: KeyModifiers::NONE,
            ..
        }) => Update::ToggleUIMode,
        Event::Key(keyevent) => match app.mode {
            UIMode::Viewer => handle_input_viewer(app, tui, keyevent),
            UIMode::SearchPanel => handle_input_search_panel(app, tui, keyevent),
        },
        Event::Resize(_, _) => Update::WindowResize(tui.size()),
        _ => Update::None,
    }
}

pub fn handle_input_viewer(app: &App, tui: &Tui, keyevent: KeyEvent) -> Update {
    match keyevent {
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Update::Quit,
        KeyEvent {
            code: KeyCode::Char('j') | KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            ..
        } => Update::ScrollViewer(1),
        KeyEvent {
            code: KeyCode::Char('k') | KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            ..
        } => Update::ScrollViewer(-1),
        KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Update::ScrollViewer((tui.size().height as f32 * 0.4).floor() as isize),
        KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Update::ScrollViewer(-(tui.size().height as f32 * 0.4).floor() as isize),
        KeyEvent {
            code: KeyCode::Char('g'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            let mut next: Event = tui.events.next().unwrap();
            while let Event::Tick = next {
                next = tui.events.next().unwrap();
            }
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('g'),
                modifiers: KeyModifiers::NONE,
                ..
            }) = next
            {
                return Update::ScrollViewer(isize::MIN + 1); // negating isize::MIN cause overflow
            } else {
                Update::None
            }
        }

        _ => Update::None,
    }
}

pub fn handle_input_search_panel(app: &App, tui: &Tui, keyevent: KeyEvent) -> Update {
    let state = app.mode.get_search_panel_state();

    // arrow keys switch input focus regardless of current focus
    if keyevent.modifiers == KeyModifiers::NONE
        && vec![KeyCode::Right, KeyCode::Left].contains(&keyevent.code)
    {
        Update::SearchPanelFocus(keyevent.code == KeyCode::Left)

    // patterns list specific keybindings
    } else if state.focus == SearchPanelFocus::PatternsList {
        match keyevent {
            KeyEvent {
                code: KeyCode::Up | KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                ..
            } => Update::CycleSearchPattern(keyevent.code == KeyCode::Up),
            KeyEvent {
                code: KeyCode::Char('d') | KeyCode::Delete | KeyCode::Enter | KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                ..
            } => match state.patterns_list_selection {
                Some(selection) => Update::EditSearchPattern(SearchPatternEdit::Delete(
                    selection,
                    keyevent.code == KeyCode::Enter,
                )),
                None => Update::Msg("No pattern selected".to_string()),
            },
            _ => Update::None,
        }
    } else if state.focus != SearchPanelFocus::PatternsList
        && keyevent.modifiers == KeyModifiers::NONE
        && keyevent.code == KeyCode::Enter
    {
        let search_string: String = app.search_panel.input_pattern.lines().join("");
        let try_color = Color::from_str(&app.search_panel.input_color.lines().join(""));
        let try_u8 = u8::from_str(&app.search_panel.input_distance.lines().join(""));
        match (try_color, try_u8) {
                                       (Ok(color), Ok(distance)) => {Update::EditSearchPattern(SearchPatternEdit::Append(SearchPattern::new(search_string, color, distance, None)))},
                                       (Err(_), Ok(_)) => {Update::Msg("Color needs to be valid hex code".to_string())},
                                       (Ok(_), Err(_)) => {Update::Msg("Edit distance needs to be valid positive integer".to_string())},
                                       (Err(_), Err(_)) => {Update::Msg("Color needs to be valid hex code, edit distance needs to be valid positive integer".to_string())},
                }

    // pass to input boxes
    } else {
        Update::SearchPanelInput(state.focus, keyevent)
    }
}
