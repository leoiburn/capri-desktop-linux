# Capri Desktop (Native Linux Port)

> **"Got a connection, choom. Signal's clear."**

An interactive desktop companion mascot and Cyberpunk AI construct rewritten natively in **Rust** for Linux (Fedora, RHEL, Arch, Debian/Ubuntu).

Originally based on [lachydotmcg/capri-desktop](https://github.com/lachydotmcg/capri-desktop), this Linux port features full alpha transparency, smooth animation physics, direct X11/XWayland cursor tracking, dynamic window stretching, and an integrated local Ollama LLM chat interface.

---

## ✨ Features

- **🦀 100% Native Rust:** Fast software renderer blitting directly into an X11/XWayland transparent canvas with zero desktop lag.
- **🎭 Complete Cyberpunk Mascot Animation Set:**
  - Embedded pixel-perfect spritesheets: `idle`, `blink`, `glitch`, `grin`, `dangle`, `mouth` (talking), `perch`, `perch_blink`, and `perch_glitch`.
  - High-precision font rasterizer rendering with embedded `ShareTechMono`.
- **🖱️ Smooth Interactive Physics:**
  - **Click & Drag:** Grab Capri with left-click to drag and reposition her on your desktop.
  - **Perch Stance:** Right-click Capri or press `P` to sit on your taskbar or window borders.
- **📐 Stretch & Dynamic Resizing:**
  - **Corner Grip:** Click & drag the bottom-right corner (`◢`) to stretch Capri bigger or smaller.
  - **Middle-Click / Shift + Drag:** Hold middle-click or `Shift + Left-Click` and drag anywhere.
  - **Mouse Scroll Wheel:** Scroll over Capri to zoom in (+10%) or out (-10%).
  - **Keyboard Shortcuts:** Press `+` / `=` to enlarge, `-` to shrink.
  - **CLI Scale Flag:** Launch at custom scales (e.g. `capri --scale 1.5` or `capri --scale 2.0`).
  - **Window Manager Native:** Hold `Alt + Right-Click Drag` (KDE Plasma).
- **💬 Dual-Interface Architecture:**
  - **Terminal REPL:** The terminal shell where `capri` is invoked serves as your interactive cyberpunk communication channel (`CHOOM > `).
  - **Desktop Window:** The companion window remains completely clean and transparent with only the animated mascot. Capri reacts with glitch animations while thinking, and mouth flap animations while answering.
- **🧠 Local AI Construct:**
  - Backed by an abliterated local LLM (e.g. Llama-3.2-3B or Qwen 30B) running locally via **Ollama**.
  - Authentic *Project Capricorn* construct system persona with multi-turn conversation memory.
- **📦 RPM Packaging & Docker Test Harness:**
  - Ready-to-install RPM spec providing `/usr/bin/capri`, desktop entry, and icon.
  - Isolated Docker test harness (`docker-test/`) with headless smoke test and virtual Xvfb test loop.

---

## 🚀 Quick Start

### 1. Prerequisites

- **Linux OS:** Fedora, CentOS Stream, RHEL, or any Linux distro with X11 / XWayland.
- **Rust Toolchain:** `rustup default stable`
- **Development Libraries:**
  ```bash
  # Fedora / RHEL
  sudo dnf install libX11-devel libXcursor-devel gcc
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

Verify that the model is responding:
```bash
ollama run capri "choom, you online?"
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

| Action | Control |
| :--- | :--- |
| **Move Mascot** | Click & Drag Capri with **Left Mouse Button** |
| **Stretch / Resize** | Drag bottom-right corner (`◢`), **Middle-Click** Drag, or **Shift + Drag** |
| **Zoom In / Out** | **Mouse Scroll Wheel** over Capri, or **`+`** / **`-`** keys |
| **Toggle Perching** | **Right-Click** on Capri, or press **`P`** |
| **Chat with Construct** | Type your prompt at the terminal prompt (`CHOOM > `) |
| **Clear Memory** | Type `clear` in terminal REPL |
| **Exit** | Type `exit` / `quit` in terminal, or press **`Esc`** on the mascot window |

---

## 🛠️ CLI Options

```
Usage: capri [OPTIONS]

Options:
  --model <NAME>        Ollama model to use (default: capri)
  --ollama-url <URL>    Ollama server URL (default: http://127.0.0.1:11434)
  --scale <FACTOR>      Initial mascot scale factor (e.g. 1.0, 1.5, 2.0; default: 1.0)
  --test-chat <PROMPT>  Headless test: send a chat prompt to Ollama, print response, exit
  --smoke-test          Headless verification of sprites, fonts, and Ollama connection
  -h, --help            Print help information
```

---

## 📦 Building the RPM Package

An RPM spec and automated build script are included:

```bash
./build_rpm.sh
```

This compiles the release binary, creates the source tarball, and runs `rpmbuild` to generate:
`rpmbuild/RPMS/x86_64/capri-desktop-1.0.0-1.fc44.x86_64.rpm`

To install on your system:
```bash
sudo dnf install capri-desktop-1.0.0-1.fc44.x86_64.rpm
```

---

## 🐳 Docker Test Environment

To test the RPM package inside an isolated headless Docker container with virtual X11 (`Xvfb`):

```bash
cd docker-test
./test_loop.sh
```

---

## 📜 License

Licensed under the [MIT License](rpmbuild/SOURCES/LICENSE).
Sprites and original character concept property of the original Capri creators.
Share Tech Mono font licensed under the [SIL Open Font License](capri-rust/assets/fonts/OFL-ShareTechMono.txt).
