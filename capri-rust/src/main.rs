mod app;
mod brain;
mod render;
mod sprites;
mod terminal;
mod tracker;

use app::App;
use brain::{Brain, DEFAULT_MODEL, DEFAULT_OLLAMA_URL};
use sprites::CapriTheme;
use std::env;
use std::sync::mpsc::channel;
use std::thread;

fn print_help() {
    println!("Capri Desktop (Native Linux Port) - Cyberpunk Construct & Desktop Mascot");
    println!("Usage: capri [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --model <NAME>        Ollama model to use (default: capri)");
    println!("  --ollama-url <URL>    Ollama server URL (default: http://127.0.0.1:11434)");
    println!("  --scale <FACTOR>      Initial mascot scale factor (e.g. 1.0, 1.5, 2.0; default: 1.0)");
    println!("  --theme <pixel|hd>    Mascot art style (pixel: retro sprite, hd: illustrated anime; default: pixel)");
    println!("  --voice               Enable TTS speech synthesis using espeak-ng / spd-say");
    println!("  --test-chat <PROMPT>  Headless test: send a chat prompt to Ollama, print response, exit");
    println!("  --smoke-test          Headless verification of sprites, fonts, and Ollama connection");
    println!("  -h, --help            Print help information");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut model = DEFAULT_MODEL.to_string();
    let mut ollama_url = DEFAULT_OLLAMA_URL.to_string();
    let mut scale: f32 = 1.0;
    let mut theme = CapriTheme::Pixel;
    let mut voice_enabled = false;
    let mut test_chat_prompt: Option<String> = None;
    let mut smoke_test = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--model" => {
                if i + 1 < args.len() {
                    model = args[i + 1].clone();
                    i += 1;
                }
            }
            "--ollama-url" => {
                if i + 1 < args.len() {
                    ollama_url = args[i + 1].clone();
                    i += 1;
                }
            }
            "--scale" => {
                if i + 1 < args.len() {
                    if let Ok(s) = args[i + 1].parse::<f32>() {
                        scale = s.clamp(0.5, 4.0);
                    }
                    i += 1;
                }
            }
            "--theme" => {
                if i + 1 < args.len() {
                    let t = args[i + 1].to_lowercase();
                    if t == "hd" || t == "anime" || t == "illustrated" {
                        theme = CapriTheme::Hd;
                    } else {
                        theme = CapriTheme::Pixel;
                    }
                    i += 1;
                }
            }
            "--voice" => {
                voice_enabled = true;
            }
            "--test-chat" => {
                if i + 1 < args.len() {
                    test_chat_prompt = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    test_chat_prompt = Some("Hey Capri, wake up and say something.".to_string());
                }
            }
            "--smoke-test" => {
                smoke_test = true;
            }
            "-h" | "--help" => {
                print_help();
                return;
            }
            _ => {}
        }
        i += 1;
    }

    let brain = Brain::new(&ollama_url, &model);

    // 1. Headless Smoke Test
    if smoke_test {
        println!("=== Capri Desktop Smoke Test ===");
        println!("1. Verifying Sprite Assets...");
        let sprites = sprites::SpriteCollection::load_embedded();
        match sprites {
            Ok(s) => {
                println!(
                    "   [PASS] Loaded Pixel Sprites (idle frames: {}, dangle frames: {}, sleep frames: {})",
                    s.idle.count, s.dangle.count, s.sleep.count
                );
                println!(
                    "   [PASS] Loaded HD Illustrated Art (idle: {} frames, blink: {} frame, happy: {} frame, sleep: {} frames)",
                    s.hd_idle.count, s.hd_blink.count, s.hd_happy.count, s.hd_sleeping.count
                );
            }
            Err(e) => {
                eprintln!("   [FAIL] Sprite loading error: {}", e);
                std::process::exit(1);
            }
        }

        println!("2. Verifying Font & Renderer...");
        match render::Renderer::new() {
            Ok(_) => println!("   [PASS] ShareTechMono font loaded and rasterizer initialized."),
            Err(e) => {
                eprintln!("   [FAIL] Font initialization error: {}", e);
                std::process::exit(1);
            }
        }

        println!(
            "3. Verifying Ollama Connectivity (Model: '{}', URL: '{}')...",
            model, ollama_url
        );
        match brain.ask_sync("Respond with 'CHOOM_ONLINE' if you can read this.") {
            Ok(reply) => {
                println!("   [PASS] Received response from Capri construct:");
                println!("          \"{}\"", reply);
            }
            Err(e) => {
                eprintln!("   [WARN] Ollama test request failed: {}", e);
                eprintln!(
                    "          (Make sure Ollama is running and model '{}' is installed)",
                    model
                );
                std::process::exit(1);
            }
        }

        println!("=== Smoke Test Passed Successfully! ===");
        return;
    }

    // 2. Headless Test Chat
    if let Some(prompt) = test_chat_prompt {
        println!("Sending prompt to Capri [Model: {}]: \"{}\"", model, prompt);
        match brain.ask_sync(&prompt) {
            Ok(reply) => {
                println!("Capri: \"{}\"", reply);
            }
            Err(e) => {
                eprintln!("Error querying Capri: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // 3. Interactive Execution: Terminal REPL + Desktop Mascot Window
    let (tx_mascot, rx_mascot) = channel::<terminal::MascotCommand>();
    let (tx_term, rx_term) = channel::<terminal::TerminalNotification>();

    // Spawn Terminal REPL on a background thread so the terminal where `capri` was run is the chat
    let brain_repl = brain.clone();
    let tx_m = tx_mascot.clone();
    let repl_thread = thread::spawn(move || {
        terminal::run_terminal_repl(brain_repl, tx_m, rx_term, voice_enabled);
    });

    // Run Mascot GUI window on the main thread
    match App::new(scale, theme) {
        Ok(mut app) => {
            if let Err(e) = app.run(rx_mascot, tx_term) {
                eprintln!("Runtime error in mascot window: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to initialize Capri application: {}", e);
        }
    }

    let _ = tx_mascot.send(terminal::MascotCommand::Exit);
    let _ = repl_thread.join();
}
