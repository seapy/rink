#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Quit,
    MoveUp,
    MoveDown,
    Select,
    Search,
    ToggleCollapse,
    CollapseAll,
    ExpandAll,
    Refresh,
    CreateSession,
    KillSession,
    RenameSession,
    RenameCategory,
    MoveSessionUp,
    MoveSessionDown,
    CycleSortMode,
    ShowHelp,
    Escape,
    // Input mode commands
    InputChar(char),
    InputBackspace,
    InputConfirm,
    // No-op
    None,
}
