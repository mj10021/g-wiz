use std::fs::read_to_string;
use bevy::prelude::{KeyCode, MouseButton, Resource, Color};
use serde_json::{json, Value};

#[derive(Resource)]
struct Settings {
    hole_delete_button: KeyCode,
    merge_delete_button: KeyCode,
    orbit_mouse_button: MouseButton,
    pan_mouse_button: MouseButton,
    extrusion_color: Color,
    extrusion_node_color: Color,
    travel_color: Color,
    save_suffix: String,
}


fn read_key(value: &Value, key: &str) -> KeyCode {
    let key = value.get(key).unwrap().as_str();
    match key {
        Some("del") | Some("delete") => KeyCode::Delete,
        Some("backspace") => KeyCode::Backspace,
        _ => panic!("invalid key option")
    }
}

fn read_mouse_button(value: &Value, key: &str) -> MouseButton {
    let button = value.get(key).unwrap().as_str();
    match button {
        Some("right") => MouseButton::Right,
        Some("left") => MouseButton::Left,
        Some("middle") => MouseButton::Middle,
        _ => panic!("invalid mouse button option")
    }
}

fn read_color(value: &Value, key: &str) -> Color {
    let color = value.get(key).unwrap().as_str().unwrap();
    Color::hex(color).unwrap()
}

fn read() -> Result<Settings, std::io::Error> {
    let settings = read_to_string("./settings.json")?;
    let settings = json!(settings);
    Ok(Settings {
        hole_delete_button: read_key(&settings, "hole_delete"),
        merge_delete_button: read_key(&settings, "merge delete"),
        orbit_mouse_button: read_mouse_button(&settings, "mouse orbit"),
        pan_mouse_button: read_mouse_button(&settings, "mouse pan"),
        extrusion_color: read_color(&settings, "extrusion color"),
        extrusion_node_color: read_color(&settings, "extrusion node color"),
        travel_color: read_color(&settings, "travel move color"),
        save_suffix: settings.get("save suffix").unwrap().to_string()
    })
}