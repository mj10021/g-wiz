use bevy::prelude::*;
use super::console::*;
#[derive(Event)]
pub enum UiEvent {
    MergeDelete,
    HoleDelete,
    MoveDisplay(bool, bool, f32),
    SelectAll,
    SetPanOrbit(bool),
    ConsoleEnter(String),
    ConsoleResponse(String),
}
#[derive(Event)]
pub enum CommandEvent {
    Subdivide(Subdivide),
    RecalcBounds,
    Draw(Draw),
    Translate(Translate),
    Rotate(Rotate),
    Scale(Rotate),
    Undo,
    Redo,
}

#[derive(Event)]
pub enum SystemEvent {
    Open,
    Save,
    SaveAs,
    RecalcBounds,
    ForceRefresh
}
