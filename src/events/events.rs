use super::console::*;
use bevy::prelude::*;
#[derive(Event)]
pub enum UiEvent {
    MergeDelete,
    HoleDelete,
    MoveDisplay(bool, bool, f32),
    SelectAll,
    Undo,
    Redo,
    SetPanOrbit(bool),
    ConsoleEnter(String),
    ConsoleResponse(String),
    CommandEnter,
}
#[derive(Event)]
pub enum CommandEvent {
    Subdivide(Subdivide),
    Draw(Draw),
    Translate(Translate),
    Rotate(Rotate),
    Scale(Rotate),
}

#[derive(Event)]
pub enum SystemEvent {
    Open,
    Save,
    SaveAs,
    RecalcBounds,
    ForceRefresh,
}
