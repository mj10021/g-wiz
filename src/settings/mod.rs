use bevy::prelude::{Color, KeyCode, MouseButton, Resource};
use serde_json::{json, Value};
use std::fs::read_to_string;

#[derive(Resource)]
pub struct Settings {
    pub hole_delete_button: KeyCode,
    pub merge_delete_button: KeyCode,
    pub orbit_mouse_button: MouseButton,
    pub pan_mouse_button: MouseButton,
    pub extrusion_color: Color,
    pub extrusion_node_color: Color,
    pub travel_color: Color,
    pub save_suffix: String,
}

fn read_key(settings: &Value, key: &str) -> KeyCode {
    let value = settings.get("keys").unwrap();
    let key = value.get(key).unwrap().as_str();
    match key {
        Some("del") | Some("delete") => KeyCode::Delete,
        Some("backspace") => KeyCode::Backspace,
        _ => panic!("invalid key option"),
    }
}

fn read_mouse_button(settings: &Value, key: &str) -> MouseButton {
    let value = settings.get("buttons").unwrap();
    let button = value.get(key).unwrap().as_str();
    match button {
        Some("right") => MouseButton::Right,
        Some("left") => MouseButton::Left,
        Some("middle") => MouseButton::Middle,
        _ => panic!("invalid mouse button option"),
    }
}

fn read_color(settings: &Value, key: &str) -> Color {
    let value = settings.get("colors").unwrap();
    let color = value.get(key).unwrap().as_str().unwrap();
    Color::hex(color).unwrap()
}

pub fn read_settings() -> Settings {
    let path = std::env::current_dir()
        .expect("could not find working directory")
        .as_path()
        .join(std::path::PathBuf::from("src/settings/settings.json"));
    let settings = read_to_string(path).unwrap();

    let settings = serde_json::from_str(&settings).unwrap();
   // panic!("{:#?}", settings);
    Settings {
        hole_delete_button: read_key(&settings, "hole delete"),
        merge_delete_button: read_key(&settings, "merge delete"),
        orbit_mouse_button: read_mouse_button(&settings, "mouse orbit"),
        pan_mouse_button: read_mouse_button(&settings, "mouse pan"),
        extrusion_color: read_color(&settings, "extrusion color"),
        extrusion_node_color: read_color(&settings, "extrusion node color"),
        travel_color: read_color(&settings, "travel move color"),
        save_suffix: settings.get("save suffix").unwrap().to_string(),
    }
}
