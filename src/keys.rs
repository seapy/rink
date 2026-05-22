use crate::app::Mode;
use crate::command::Command;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Korean Dvorak normalization map
/// Maps Korean characters that might be typed on a Dvorak layout to their intended ASCII equivalents
fn normalize_korean_dvorak(c: char) -> Option<char> {
    match c {
        'ㅂ' => Some('q'),
        'ㅈ' => Some('w'),
        'ㄷ' => Some('e'),
        'ㄱ' => Some('r'),
        'ㅅ' => Some('t'),
        'ㅛ' => Some('y'),
        'ㅕ' => Some('u'),
        'ㅑ' => Some('i'),
        'ㅐ' => Some('o'),
        'ㅔ' => Some('p'),
        'ㅁ' => Some('a'),
        'ㄴ' => Some('s'),
        'ㅇ' => Some('d'),
        'ㄹ' => Some('f'),
        'ㅎ' => Some('g'),
        'ㅗ' => Some('h'),
        'ㅓ' => Some('j'),
        'ㅏ' => Some('k'),
        'ㅣ' => Some('l'),
        'ㅋ' => Some('z'),
        'ㅌ' => Some('x'),
        'ㅊ' => Some('c'),
        'ㅍ' => Some('v'),
        'ㅠ' => Some('b'),
        'ㅜ' => Some('n'),
        'ㅡ' => Some('m'),
        _ => None,
    }
}

pub fn translate(key: KeyEvent, mode: &Mode) -> Command {
    // Handle input modes (search, create, rename)
    match mode {
        Mode::Search | Mode::Creating | Mode::Renaming | Mode::RenamingCategory => {
            return translate_input_mode(key);
        }
        Mode::ConfirmKill => {
            return translate_confirm_mode(key);
        }
        _ => {}
    }

    // Normal / Help mode
    match key.code {
        KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => Command::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Command::Quit,
        KeyCode::Up | KeyCode::Char('k') => Command::MoveUp,
        KeyCode::Down | KeyCode::Char('j') => Command::MoveDown,
        KeyCode::Enter => Command::Select,
        KeyCode::Char('/') => Command::Search,
        KeyCode::Tab => Command::ToggleCollapse,
        KeyCode::Left => Command::ToggleCollapse,
        KeyCode::Right => Command::ToggleCollapse,
        KeyCode::Char('r') if !key.modifiers.contains(KeyModifiers::SHIFT) => Command::Refresh,
        KeyCode::Char('c') if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            Command::CreateSession
        }
        KeyCode::Char('x') => Command::KillSession,
        KeyCode::Char('R') => Command::RenameSession,
        KeyCode::Char('C') => Command::RenameCategory,
        KeyCode::Char('J') => Command::MoveSessionDown,
        KeyCode::Char('K') => Command::MoveSessionUp,
        KeyCode::Char('s') => Command::CycleSortMode,
        KeyCode::Char('?') => Command::ShowHelp,
        KeyCode::Esc => Command::Escape,
        // Korean normalization
        KeyCode::Char(c) => {
            if let Some(ascii) = normalize_korean_dvorak(c) {
                translate(KeyEvent::new(KeyCode::Char(ascii), key.modifiers), mode)
            } else {
                Command::None
            }
        }
        _ => Command::None,
    }
}

fn translate_input_mode(key: KeyEvent) -> Command {
    match key.code {
        KeyCode::Esc => Command::Escape,
        KeyCode::Enter => Command::InputConfirm,
        KeyCode::Backspace => Command::InputBackspace,
        KeyCode::Char(c) => Command::InputChar(c),
        _ => Command::None,
    }
}

fn translate_confirm_mode(key: KeyEvent) -> Command {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => Command::InputConfirm,
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Command::Escape,
        _ => Command::None,
    }
}
