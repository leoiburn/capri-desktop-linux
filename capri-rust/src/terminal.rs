use std::io::{self, Write};
use std::sync::mpsc::{Receiver, Sender};

use crate::brain::Brain;

pub enum MascotCommand {
    Thinking,
    Talking(f64), // Duration in seconds
    Idle,
    Exit,
}

pub enum TerminalNotification {
    MascotClosed,
}

pub fn run_terminal_repl(
    brain: Brain,
    tx_mascot: Sender<MascotCommand>,
    rx_term: Receiver<TerminalNotification>,
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
    println!("\x1b[2;37m* Move: Click & drag Capri to toss/reposition her on your screen.\x1b[0m");
    println!("\x1b[2;37m* Stance: Right-Click or press 'P' to toggle Perching stance.\x1b[0m");
    println!("\x1b[2;37m* Type your message below to chat. Type 'exit' to disconnect.\x1b[0m\n");

    let stdin = io::stdin();
    let mut input_buffer = String::new();

    loop {
        // Check if the mascot window was closed
        if let Ok(TerminalNotification::MascotClosed) = rx_term.try_recv() {
            println!("\r\x1b[2;36m[Mascot window closed. Disconnecting Capri...]\x1b[0m");
            break;
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

                // Notify mascot to show thinking / expectant reaction
                let _ = tx_mascot.send(MascotCommand::Thinking);

                print!("\x1b[2;33m[Capri is thinking...]\x1b[0m");
                let _ = io::stdout().flush();

                match brain.ask(trimmed) {
                    Ok(reply) => {
                        // Calculate speech animation duration based on word count
                        let word_count = reply.split_whitespace().count();
                        let duration = (word_count as f64 * 0.26).clamp(2.5, 9.0);

                        // Trigger mouth flap animation on desktop mascot
                        let _ = tx_mascot.send(MascotCommand::Talking(duration));

                        // Erase "thinking..." indicator and print Capri's reply
                        print!("\r\x1b[2K");
                        println!("\x1b[1;36mCAPRI >\x1b[0m \x1b[1;37m\"{}\"\x1b[0m\n", reply);
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
