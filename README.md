# Capri Desktop (Native Linux Port)

> **"Got a connection, choom. Signal's clear."**

An interactive desktop companion mascot and Cyberpunk AI construct rewritten natively in **Rust** for Linux (Fedora, RHEL, Arch, Debian/Ubuntu).

Combining features from both the original [lachydotmcg/capri-desktop](https://github.com/lachydotmcg/capri-desktop) and the Python desktop companion [Juliowiwiwiwi/Capri-Capricorn-The-Desktop-Construct](https://github.com/Juliowiwiwiwi/Capri-Capricorn-The-Desktop-Construct), this Linux port features full alpha transparency, smooth physics, direct X11/XWayland cursor tracking, dynamic window stretching, dual art themes (Pixel Art & HD Illustrated Anime), auto-sleep/wake cycles, poke interactions, real-time token streaming, and an integrated local Ollama LLM chat interface with optional voice synthesis.

---

## ✨ Features

- **🦀 100% Native Rust:** Fast software renderer blitting directly into an X11/XWayland transparent canvas with zero desktop lag.
- **🎨 Dual Mascot Themes (Pixel & HD Illustrated):**
  - **Pixel Art Theme:** Classic Cyberpunk retro spritesheets (`idle`, `blink`, `glitch`, `grin`, `dangle`, `mouth`, `perch`, `sleep`, `wake`).
  - **HD Illustrated Theme:** Crisp high-definition illustrated character art (`capri_idle1/2/3`, `capri_blink`, `capri_happy`, `capri_sleeping0/1/2`).
  - Switch anytime at runtime by pressing **`T`** or launch with `--theme hd` / `--theme pixel`.
- **💤 Inactivity Sleep & Wake Cycles:**
  - **Auto-Sleep:** After 60 seconds of inactivity, Capri automatically yawns and transitions to her sleeping animation loop.
  - **Wake Up:** Double-clicking Capri, clicking while asleep, sending a chat message, or pressing **`W`** instantly wakes her up with a happy reaction.
  - **Manual Sleep:** Press **`S`** or type `/sleep` in the terminal anytime.
- **✨ Poke Interaction:**
  - Double-clicking Capri pokes her, triggering her joyful/happy reaction pose and resetting her sleep timer.
- **⚡ Real-Time Async Token Streaming:**
  - Responses from Ollama stream live token-by-token directly into your terminal in real-time.
  - Capri's mouth flaps continuously while streaming tokens, returning to idle when complete.
- **🔊 Cyberpunk Voice Synthesis (TTS):**
  - Optional text-to-speech engine using `espeak-ng` / `spd-say` (enable with `--voice` or type `/voice on` in the terminal).
- **🖱️ Smooth Interactive Physics:**
  - **Click & Drag:** Grab Capri with left-click to drag and reposition her on your desktop.
  - **Perch Stance:** Right-click Capri or press `P` to sit on your taskbar or window borders.
- **📐 Dynamic Window Stretching & Resizing:**
  - **Corner Grip:** Click & drag the bottom-right corner (`◢`) to stretch Capri bigger or smaller.
  - **Middle-Click / Shift + Drag:** Hold middle-click or `Shift + Left-Click` and drag anywhere.
  - **Mouse Scroll Wheel:** Scroll over Capri to zoom in (+10%) or out (-10%).
  - **Keyboard Shortcuts:** Press `+` / `=` to enlarge, `-` to shrink.
  - **CLI Scale Flag:** Launch at custom scales (e.g. `capri --scale 1.5` or `capri --scale 2.0`).
  - **Window Manager Native:** Hold `Alt + Right-Click Drag` (KDE Plasma).
- **💬 Dual-Interface Architecture:**
  - **Terminal REPL:** The terminal shell where `capri` is invoked serves as your interactive cyberpunk communication channel (`CHOOM > `).
  - **Desktop Window:** The companion window remains completely clean and transparent with only the animated mascot.

---

## 🚀 Quick Start

### 1. Prerequisites

- **Linux OS:** Fedora, CentOS Stream, RHEL, Arch, Debian/Ubuntu with X11 / XWayland.
- **Rust Toolchain:** `rustup default stable`
- **Development Libraries:**
  ```bash
  # Fedora / RHEL
  sudo dnf install libX11-devel libXcursor-devel gcc espeak-ng
  ```
- **Ollama:** Installed and listening on `http://127.0.0.1:11434`.

### 2. Set Up the Capri Ollama Model

A pre-configured [`Modelfile.capri`](Modelfile.capri) is included in the repository.

```bash
# Pull the base abliterated model
ollama pull QuantFactory/Llama-3.2-3B-Instruct-abliterated-GGUF:Q3_K_M

# Create the Capri construct
ollama create capri -f Modelfile.capri
```

### 3. Build & Run

```bash
# Clone the repository
git clone https://github.com/leoiburn/capri-desktop-linux.git
cd capri-desktop-linux

# Build release binary
cargo build --release --manifest-path capri-rust/Cargo.toml

# Install locally
install -m 0755 capri-rust/target/release/capri ~/.local/bin/capri

# Launch Capri!
capri
```

---

## 🎮 Controls & Keybindings

### Mascot Window Controls
| Action | Control |
| :--- | :--- |
| **Move Mascot** | Click & Drag Capri with **Left Mouse Button** |
| **Poke (Happy Reaction)** | **Double Left-Click** on Capri |
| **Toggle Art Theme** | Press **`T`** (Pixel Art <-> HD Illustrated) |
| **Sleep** | Press **`S`** (Auto-triggers after 60s of inactivity) |
| **Wake Up** | Press **`W`**, single-click while sleeping, or chat |
| **Toggle Perching** | **Right-Click** on Capri, or press **`P`** |
| **Stretch / Resize** | Drag bottom-right corner (`◢`), **Middle-Click** Drag, or **Shift + Drag** |
| **Zoom In / Out** | **Mouse Scroll Wheel** over Capri, or **`+`** / **`-`** keys |
| **Exit** | Press **`Esc`** on the mascot window |

### Terminal REPL Commands
| Command | Action |
| :--- | :--- |
| **`<your prompt>`** | Send message to Capri with real-time token streaming |
| **`/theme`** | Toggle between Pixel Art and HD Illustrated art |
| **`/sleep`** | Put Capri to sleep |
| **`/wake`** | Wake Capri up |
| **`/poke`** | Poke Capri (triggers happy expression) |
| **`/voice`** | Toggle TTS voice synthesis (`on` / `off`) |
| **`clear`** | Wipe conversation memory clean |
| **`exit`** / **`quit`** | Disconnect and close companion |

---

## 🛠️ CLI Options

```
Usage: capri [OPTIONS]

Options:
  --model <NAME>        Ollama model to use (default: capri)
  --ollama-url <URL>    Ollama server URL (default: http://127.0.0.1:11434)
  --scale <FACTOR>      Initial mascot scale factor (e.g. 1.0, 1.5, 2.0; default: 1.0)
  --theme <pixel|hd>    Mascot art style (pixel: retro sprite, hd: illustrated anime; default: pixel)
  --voice               Enable TTS voice synthesis using espeak-ng / spd-say
  --test-chat <PROMPT>  Headless test: send a chat prompt to Ollama, print response, exit
  --smoke-test          Headless verification of sprites, fonts, and Ollama connection
  -h, --help            Print help information
```

---

## 📦 Building the RPM Package

```bash
./build_rpm.sh
```

This compiles the release binary, packages the source tarball, and runs `rpmbuild` to generate:
`rpmbuild/RPMS/x86_64/capri-desktop-1.0.0-1.fc44.x86_64.rpm`

To install on your system:
```bash
sudo dnf install capri-desktop-1.0.0-1.fc44.x86_64.rpm
```

---

## 🐳 Docker Test Environment

```bash
cd docker-test
./test_loop.sh
```

---

## 📜 License

Licensed under the [MIT License](rpmbuild/SOURCES/LICENSE).
Sprites and original character concept property of the original Capri creators.
Share Tech Mono font licensed under the [SIL Open Font License](capri-rust/assets/fonts/OFL-ShareTechMono.txt).
