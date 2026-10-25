use std::io::{self, Write};
use std::sync::mpsc::{Receiver, Sender};

use crate::brain::Brain;

pub enum MascotCommand {
    Thinking,
    Talking(f64), // Duration in seconds
    Idle,
    Poke,
    Sleep,
    Wake,
    ToggleTheme,
    Exit,
}

pub enum TerminalNotification {
    MascotClosed,
    Sleeping,
    Woken,
}

fn speak_text(text: &str) {
    let clean: String = text
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == ',' || *c == '.' || *c == '!' || *c == '?')
        .collect();
    if clean.trim().is_empty() {
        return;
    }
    std::thread::spawn(move || {
        let status = std::process::Command::new("espeak-ng")
            .args(&["-v", "en-us+f3", "-s", "170", &clean])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        if status.is_err() || !status.unwrap().success() {
            let _ = std::process::Command::new("spd-say")
                .args(&["-t", "female2", &clean])
                .status();
        }
    });
}

pub fn run_terminal_repl(
    brain: Brain,
    tx_mascot: Sender<MascotCommand>,
    rx_term: Receiver<TerminalNotification>,
    mut voice_enabled: bool,
) {
    println!("\x1b[1;36m╔═══════════════════════════════════════════════════════════════╗\x1b[0m");
    println!("\x1b[1;36m║\x1b[0m   \x1b[1;33mCAPRI CONSTRUCT // NIGHT CITY TERMINAL\x1b[0m                     \x1b[1;36m║\x1b[0m");
    println!(
        "\x1b[1;36m║\x1b[0m   Neural Engram: \x1b[1;32mONLINE\x1b[0m // Model: \x1b[1;35m{:<22}\x1b[0m \x1b[1;36m║\x1b[0m",
        brain.model
    );
    println!("\x1b[1;36m╚═══════════════════════════════════════════════════════════════╝\x1b[0m");
    println!("\x1b[2;37m* Mascot window is active on your desktop.\x1b[0m");
    println!("\x1b[2;37m* Stretch/Resize: Drag bottom-right corner, Middle-Click drag, Shift+Drag, Scroll Wheel, or +/-.\x1b[0m");
    println!("\x1b[2;37m* Interactions: Click & drag Capri to toss her | Double-Click to Poke | 'P' to Perch.\x1b[0m");
    println!("\x1b[2;37m* Theme: Press 'T' or type /theme to switch between Pixel Art & HD Illustrated Art.\x1b[0m");
    println!("\x1b[2;37m* Sleep: Inactivity timeout after 60s | Press 'S' to sleep | 'W' to wake.\x1b[0m");
    println!("\x1b[2;37m* Voice: Type /voice to toggle TTS speech synthesis [{}]\x1b[0m", if voice_enabled { "ON" } else { "OFF" });
    println!("\x1b[2;37m* Commands: /sleep, /wake, /poke, /theme, /voice, clear, exit.\x1b[0m\n");

    let stdin = io::stdin();
    let mut input_buffer = String::new();

    loop {
        // Check notifications from the mascot window
        while let Ok(notif) = rx_term.try_recv() {
            match notif {
                TerminalNotification::MascotClosed => {
                    println!("\r\x1b[2;36m[Mascot window closed. Disconnecting Capri...]\x1b[0m");
                    return;
                }
                TerminalNotification::Sleeping => {
                    println!("\r\x1b[2;35m[Capri dozed off to sleep... (Double-click her, press W, or chat to wake)]\x1b[0m");
                    print!("\x1b[1;33mCHOOM > \x1b[0m");
                    let _ = io::stdout().flush();
                }
                TerminalNotification::Woken => {
                    println!("\r\x1b[2;32m[Capri woke up! \"Hey choom, what's good?\"]\x1b[0m");
                    print!("\x1b[1;33mCHOOM > \x1b[0m");
                    let _ = io::stdout().flush();
                }
            }
        }

        print!("\x1b[1;33mCHOOM > \x1b[0m");
        let _ = io::stdout().flush();

        input_buffer.clear();
        match stdin.read_line(&mut input_buffer) {
            Ok(0) => {
                // EOF (Ctrl+D)
                println!();
                let _ = tx_mascot.send(MascotCommand::Exit);
                break;
            }
            Ok(_) => {
                let trimmed = input_buffer.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed.eq_ignore_ascii_case("exit")
                    || trimmed.eq_ignore_ascii_case("quit")
                    || trimmed.eq_ignore_ascii_case(":q")
                {
                    let _ = tx_mascot.send(MascotCommand::Exit);
                    println!("\x1b[2;36m[Connection terminated. Capri construct disconnected.]\x1b[0m");
                    break;
                }
                if trimmed.eq_ignore_ascii_case("clear") {
                    brain.clear_history();
                    println!("\x1b[2;33m[Conversation memory wiped clean.]\x1b[0m\n");
                    continue;
                }
                if trimmed.eq_ignore_ascii_case("/sleep") {
                    let _ = tx_mascot.send(MascotCommand::Sleep);
                    println!("\x1b[2;35m[Told Capri to sleep. 💤]\x1b[0m\n");
                    continue;
                }
                if trimmed.eq_ignore_ascii_case("/wake") {
                    let _ = tx_mascot.send(MascotCommand::Wake);
                    println!("\x1b[2;32m[Capri is awake! ⚡]\x1b[0m\n");
                    continue;
                }
                if trimmed.eq_ignore_ascii_case("/poke") {
                    let _ = tx_mascot.send(MascotCommand::Poke);
                    println!("\x1b[2;33m[You poked Capri! ✨]\x1b[0m\n");
                    continue;
                }
                if trimmed.eq_ignore_ascii_case("/theme") {
                    let _ = tx_mascot.send(MascotCommand::ToggleTheme);
                    println!("\x1b[2;36m[Toggled mascot art theme (Pixel <-> HD Illustrated).]\x1b[0m\n");
                    continue;
                }
                if trimmed.eq_ignore_ascii_case("/voice") {
                    voice_enabled = !voice_enabled;
                    println!(
                        "\x1b[2;36m[Voice synthesis toggled: {}]\x1b[0m\n",
                        if voice_enabled { "\x1b[1;32mENABLED\x1b[2;36m" } else { "\x1b[1;31mDISABLED\x1b[2;36m" }
                    );
                    continue;
                }
                if trimmed.eq_ignore_ascii_case("/voice on") {
                    voice_enabled = true;
                    println!("\x1b[2;32m[Voice synthesis ENABLED]\x1b[0m\n");
                    continue;
                }
                if trimmed.eq_ignore_ascii_case("/voice off") {
                    voice_enabled = false;
                    println!("\x1b[2;31m[Voice synthesis DISABLED]\x1b[0m\n");
                    continue;
                }
                if trimmed.eq_ignore_ascii_case("/help") || trimmed.eq_ignore_ascii_case("help") {
                    println!("\x1b[1;36mCommands:\x1b[0m");
                    println!("  /theme      Toggle between Pixel Art and HD Illustrated Character");
                    println!("  /sleep      Put Capri to sleep (plays sleeping animation)");
                    println!("  /wake       Wake Capri up");
                    println!("  /poke       Poke Capri (triggers happy reaction pose)");
                    println!("  /voice      Toggle TTS voice synthesis (on/off)");
                    println!("  clear       Clear conversation history");
                    println!("  exit        Disconnect and quit");
                    println!();
                    println!("\x1b[1;36mHotkeys on Mascot Window:\x1b[0m");
                    println!("  T           Toggle Pixel / HD theme");
                    println!("  P           Toggle Perching stance");
                    println!("  S           Sleep");
                    println!("  W           Wake up");
                    println!("  +/-         Zoom / stretch size");
                    println!("  Double-Click Poke Capri\n");
                    continue;
                }

                // Notify mascot to wake up and show thinking
                let _ = tx_mascot.send(MascotCommand::Wake);
                let _ = tx_mascot.send(MascotCommand::Thinking);

                print!("\x1b[2;33m[Capri is thinking...]\x1b[0m");
                let _ = io::stdout().flush();

                let mut first_token = true;
                let tx_m = tx_mascot.clone();

                let stream_res = brain.ask_stream(trimmed, |tok| {
                    if first_token {
                        first_token = false;
                        print!("\r\x1b[2K\x1b[1;36mCAPRI >\x1b[0m \x1b[1;37m\"");
                        let _ = tx_m.send(MascotCommand::Talking(15.0));
                    }
                    print!("{}", tok);
                    let _ = io::stdout().flush();
                });

                match stream_res {
                    Ok(reply) => {
                        if first_token {
                            print!("\r\x1b[2K\x1b[1;36mCAPRI >\x1b[0m \x1b[1;37m\"");
                            print!("{}", reply);
                        }
                        println!("\"\x1b[0m\n");
                        let _ = tx_mascot.send(MascotCommand::Idle);

                        if voice_enabled {
                            speak_text(&reply);
                        }
                    }
                    Err(e) => {
                        let _ = tx_mascot.send(MascotCommand::Idle);
                        print!("\r\x1b[2K");
                        println!("\x1b[1;31m[SIGNAL CORRUPTED]\x1b[0m {}\n", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("\nError reading input: {}", e);
                let _ = tx_mascot.send(MascotCommand::Exit);
                break;
            }
        }
    }
}
