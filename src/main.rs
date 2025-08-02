use enigo::{Button, Coordinate::Abs, Direction::Click, Enigo, Mouse, Settings};
use std::io::{self, Read, Write};

fn read_message() -> Option<String> {
    let mut len_buf = [0; 4];
    io::stdin().read_exact(&mut len_buf).ok()?;
    let len = u32::from_le_bytes(len_buf) as usize;

    let mut msg_buf = vec![0; len];
    io::stdin().read_exact(&mut msg_buf).ok()?;
    String::from_utf8(msg_buf).ok()
}

fn send_message(response: &str) {
    let bytes = response.as_bytes();
    let len = (bytes.len() as u32).to_le_bytes();
    io::stdout().write_all(&len).unwrap();
    io::stdout().write_all(bytes).unwrap();
    io::stdout().flush().unwrap();
}

fn main() {
    if let Some(input) = read_message() {
        // parsing stdin
        let parsed: serde_json::Value = serde_json::from_str(&input).unwrap_or_default();
        let x = parsed.get("x").and_then(|v| v.as_i64()).unwrap();
        let y = parsed.get("y").and_then(|v| v.as_i64()).unwrap();

        // mouse manipulation
        let mut enigo = Enigo::new(&Settings::default()).unwrap();
        enigo.move_mouse(x as i32, y as i32, Abs).unwrap();
        enigo.button(Button::Left, Click).unwrap();

        // success report
        send_message(&format!("clicked to ({}, {})", x, y));
    } else {
        eprintln!("No stdin with click coordinates provided");
    }
}
