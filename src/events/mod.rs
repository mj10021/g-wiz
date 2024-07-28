pub mod console;
pub mod handlers;

use console::*;


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
    Translate(Translate),
    Rotate(Rotate),
    Scale(Scale),
    Subdivide(Subdivide),
    Draw(Draw),
    Filter(Filter),
    Map(Map),
}

#[derive(Event)]
pub enum SystemEvent {
    Open,
    Save,
    SaveAs,
    RecalcBounds,
    ForceRefresh,
}
