mod install;

use enigo::{Button, Coordinate::Abs, Direction::Click, Enigo, Mouse, Settings};
use std::{
    io::{self, IsTerminal, Read, Write},
    thread::sleep,
    time::Duration,
};

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
    // Run installer when launched manually (double-click or --install flag).
    // Chrome calls the binary via a pipe, so stdin won't be a TTY in that case.
    let is_manual = io::stdin().is_terminal() || std::env::args().any(|a| a == "--install");

    if is_manual {
        install::install();
        return;
    }

    if let Some(input) = read_message() {
        let parsed: serde_json::Value = serde_json::from_str(&input).unwrap_or_default();

        let x = parsed.get("x").and_then(|v| v.as_i64());
        let y = parsed.get("y").and_then(|v| v.as_i64());

        match (x, y) {
            (Some(x), Some(y)) => {
                let mut enigo = Enigo::new(&Settings::default()).unwrap();
                enigo.move_mouse(x as i32, y as i32, Abs).unwrap();
                sleep(Duration::from_millis(100));
                enigo.button(Button::Left, Click).unwrap();
                send_message(&format!("clicked to ({}, {})", x, y));
            }
            _ => {
                send_message("{\"ready\": true}");
            }
        }
    }
}
