use minifb::{Key, MouseButton, MouseMode, ScaleMode, Window, WindowOptions};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

use crate::render::Renderer;
use crate::sprites::{CapriTheme, Clip, SpriteCollection};
use crate::terminal::{MascotCommand, TerminalNotification};
use crate::tracker::X11CursorTracker;

pub const BUF_WIDTH: usize = 512;
pub const BUF_HEIGHT: usize = 384;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum CapriState {
    Idle,
    Blink,
    Glitch,
    Grin,       // Happy / Poke reaction
    Dangle,
    Talking,
    Perch,
    PerchBlink,
    PerchGlitch,
    Sleeping,   // Inactivity sleep
    Wake,       // Waking up animation
}

pub struct App {
    pub window: Window,
    pub buffer: Vec<u32>,
    pub sprites: SpriteCollection,
    pub tracker: Option<X11CursorTracker>,
    pub theme: CapriTheme,
    pub state: CapriState,
    pub state_start: Instant,
    pub next_blink: Instant,
    pub next_quirk: Instant,
    pub talking_until: Option<Instant>,
    pub last_interaction: Instant,
    pub last_left_click: Option<Instant>,
    pub click_latch: bool,
    pub is_perched: bool,
    pub dragging: bool,
    pub resizing: bool,
    pub resize_start_cursor: (isize, isize),
    pub resize_start_size: (u32, u32),
    pub right_down: bool,
    pub drag_offset: (isize, isize),
    pub win_pos: (isize, isize),
    pub win_size: (u32, u32),
}

impl App {
    pub fn new(scale: f32, theme: CapriTheme) -> Result<Self, String> {
        let initial_w = ((BUF_WIDTH as f32) * scale).round() as usize;
        let initial_h = ((BUF_HEIGHT as f32) * scale).round() as usize;

        let opts = WindowOptions {
            borderless: true,
            title: false,
            resize: true,
            scale_mode: ScaleMode::Stretch,
            topmost: true,
            transparency: true,
            ..WindowOptions::default()
        };

        let mut window = Window::new("Capri", initial_w, initial_h, opts)
            .map_err(|e| format!("Failed to create window: {}", e))?;

        window.set_target_fps(60);

        let buffer = vec![0u32; BUF_WIDTH * BUF_HEIGHT];
        let sprites = SpriteCollection::load_embedded()?;
        let tracker = X11CursorTracker::new();

        let now = Instant::now();

        let initial_x = 450;
        let initial_y = 350;
        window.set_position(initial_x, initial_y);

        Ok(App {
            window,
            buffer,
            sprites,
            tracker,
            theme,
            state: CapriState::Idle,
            state_start: now,
            next_blink: now + Duration::from_millis(3500),
            next_quirk: now + Duration::from_secs(8),
            talking_until: None,
            last_interaction: now,
            last_left_click: None,
            click_latch: false,
            is_perched: false,
            dragging: false,
            resizing: false,
            resize_start_cursor: (0, 0),
            resize_start_size: (initial_w as u32, initial_h as u32),
            right_down: false,
            drag_offset: (0, 0),
            win_pos: (initial_x, initial_y),
            win_size: (initial_w as u32, initial_h as u32),
        })
    }

    pub fn poke(&mut self, now: Instant) {
        self.state = CapriState::Grin;
        self.state_start = now;
        self.last_interaction = now;
    }

    pub fn wake_up(&mut self, now: Instant, tx_term: &Sender<TerminalNotification>) {
        self.state = CapriState::Wake;
        self.state_start = now;
        self.last_interaction = now;
        let _ = tx_term.send(TerminalNotification::Woken);
    }

    pub fn go_to_sleep(&mut self, now: Instant, tx_term: &Sender<TerminalNotification>) {
        self.state = CapriState::Sleeping;
        self.state_start = now;
        let _ = tx_term.send(TerminalNotification::Sleeping);
    }

    pub fn run(
        &mut self,
        rx_cmd: Receiver<MascotCommand>,
        tx_term: Sender<TerminalNotification>,
    ) -> Result<(), String> {
        while self.window.is_open() {
            let now = Instant::now();

            // Quit on Esc
            if self.window.is_key_pressed(Key::Escape, minifb::KeyRepeat::No) {
                let _ = tx_term.send(TerminalNotification::MascotClosed);
                break;
            }

            // Check messages from Terminal REPL
            while let Ok(cmd) = rx_cmd.try_recv() {
                self.last_interaction = now;
                match cmd {
                    MascotCommand::Thinking => {
                        self.state = CapriState::Glitch;
                        self.state_start = now;
                    }
                    MascotCommand::Talking(duration) => {
                        self.state = CapriState::Talking;
                        self.state_start = now;
                        self.talking_until = Some(now + Duration::from_secs_f64(duration));
                    }
                    MascotCommand::Idle => {
                        self.state = if self.is_perched { CapriState::Perch } else { CapriState::Idle };
                        self.state_start = now;
                        self.talking_until = None;
                    }
                    MascotCommand::Poke => {
                        self.poke(now);
                    }
                    MascotCommand::Sleep => {
                        self.go_to_sleep(now, &tx_term);
                    }
                    MascotCommand::Wake => {
                        if self.state == CapriState::Sleeping {
                            self.wake_up(now, &tx_term);
                        }
                    }
                    MascotCommand::ToggleTheme => {
                        self.theme = match self.theme {
                            CapriTheme::Pixel => CapriTheme::Hd,
                            CapriTheme::Hd => CapriTheme::Pixel,
                        };
                    }
                    MascotCommand::Exit => {
                        return Ok(());
                    }
                }
            }

            self.handle_input(now, &tx_term);
            self.update_state(now, &tx_term);
            self.render(now);

            self.window
                .update_with_buffer(&self.buffer, BUF_WIDTH, BUF_HEIGHT)
                .map_err(|e| format!("Failed to update window: {}", e))?;
        }

        let _ = tx_term.send(TerminalNotification::MascotClosed);
        Ok(())
    }

    fn handle_input(&mut self, now: Instant, tx_term: &Sender<TerminalNotification>) {
        // 1. Sync win_size with actual window dimensions (e.g. if resized by KDE Alt+Right-Click)
        let (actual_w, actual_h) = self.window.get_size();
        if actual_w > 0 && actual_h > 0 {
            if actual_w as u32 != self.win_size.0 || actual_h as u32 != self.win_size.1 {
                self.win_size = (actual_w as u32, actual_h as u32);
            }
        }

        // 2. Normalize mouse position to 512x384 buffer coordinates
        let scale_factor = ((self.win_size.0 as f32) / (BUF_WIDTH as f32))
            .min((self.win_size.1 as f32) / (BUF_HEIGHT as f32))
            .max(0.1);

        let mouse_pos = self.window.get_mouse_pos(MouseMode::Pass);
        let mouse_buf = mouse_pos.map(|(mx, my)| (mx / scale_factor, my / scale_factor));

        // Keyboard hotkeys
        if self.window.is_key_pressed(Key::T, minifb::KeyRepeat::No) {
            self.theme = match self.theme {
                CapriTheme::Pixel => CapriTheme::Hd,
                CapriTheme::Hd => CapriTheme::Pixel,
            };
            self.last_interaction = now;
        }

        if self.window.is_key_pressed(Key::S, minifb::KeyRepeat::No) {
            self.go_to_sleep(now, tx_term);
        }

        if self.window.is_key_pressed(Key::W, minifb::KeyRepeat::No) {
            self.wake_up(now, tx_term);
        }

        // Toggle Perch with Right Mouse Button (click event)
        let right_pressed = self.window.get_mouse_down(MouseButton::Right);
        if right_pressed && !self.right_down && !self.dragging && !self.resizing {
            self.right_down = true;
            self.is_perched = !self.is_perched;
            self.state = if self.is_perched { CapriState::Perch } else { CapriState::Idle };
            self.state_start = now;
            self.last_interaction = now;
        } else if !right_pressed {
            self.right_down = false;
        }

        // Toggle Perch with 'P' key
        if self.window.is_key_pressed(Key::P, minifb::KeyRepeat::No) {
            self.is_perched = !self.is_perched;
            self.state = if self.is_perched { CapriState::Perch } else { CapriState::Idle };
            self.state_start = now;
            self.last_interaction = now;
        }

        // Window Resizing via Mouse Scroll Wheel
        if let Some((_, scroll_y)) = self.window.get_scroll_wheel() {
            if scroll_y > 0.5 {
                self.adjust_size(1.10);
            } else if scroll_y < -0.5 {
                self.adjust_size(0.90);
            }
        }

        // Window Resizing via Keyboard '+' or '-' / '[' or ']'
        if self.window.is_key_pressed(Key::Equal, minifb::KeyRepeat::Yes)
            || self.window.is_key_pressed(Key::RightBracket, minifb::KeyRepeat::Yes)
        {
            self.adjust_size(1.10);
        }
        if self.window.is_key_pressed(Key::Minus, minifb::KeyRepeat::Yes)
            || self.window.is_key_pressed(Key::LeftBracket, minifb::KeyRepeat::Yes)
        {
            self.adjust_size(0.90);
        }

        // Mouse buttons and keys
        let left_down = self.window.get_mouse_down(MouseButton::Left);
        let middle_down = self.window.get_mouse_down(MouseButton::Middle);
        let shift_down = self.window.is_key_down(Key::LeftShift) || self.window.is_key_down(Key::RightShift);

        // Character bounds in buffer coordinates
        let (sprite_x, sprite_w) = match self.theme {
            CapriTheme::Pixel => {
                if self.is_perched { (64.0, 384.0) } else { (136.0, 240.0) }
            }
            CapriTheme::Hd => (60.0, 392.0),
        };
        let sprite_h = 384.0;

        let over_capri = if let Some((bx, by)) = mouse_buf {
            bx >= sprite_x && bx <= (sprite_x + sprite_w) && by >= 0.0 && by <= sprite_h
        } else {
            false
        };

        // Corner resize grip (bottom-right 50x50 area)
        let over_corner = if let Some((bx, by)) = mouse_buf {
            bx >= 460.0 && by >= 330.0
        } else {
            false
        };

        // Resizing initiation (Middle drag, Shift+Left drag, or Corner drag)
        let wants_resize = middle_down || (left_down && (shift_down || over_corner));

        if wants_resize && !self.resizing && !self.dragging {
            if let Some(ref tracker) = self.tracker {
                if let Some(cursor) = tracker.get_global_cursor() {
                    self.resizing = true;
                    self.resize_start_cursor = cursor;
                    self.resize_start_size = self.win_size;
                }
            }
        }

        // Resizing execution
        if self.resizing {
            let still_resizing = middle_down || left_down;
            if still_resizing {
                if let Some(ref tracker) = self.tracker {
                    if let Some(cursor) = tracker.get_global_cursor() {
                        let dx = cursor.0 - self.resize_start_cursor.0;
                        let dy = cursor.1 - self.resize_start_cursor.1;
                        let delta = if dx.abs() > dy.abs() { dx } else { dy };
                        let new_w = ((self.resize_start_size.0 as isize + delta).clamp(180, 1600)) as u32;
                        let new_h = ((new_w as f32 * (BUF_HEIGHT as f32 / BUF_WIDTH as f32)).round() as u32).clamp(140, 1200);
                        self.set_size(new_w, new_h);
                    }
                }
            } else {
                self.resizing = false;
            }
        }

        // Double-click / Click detection for Poke and Wake
        if left_down && !self.click_latch {
            self.click_latch = true;
            if over_capri && !shift_down && !over_corner {
                let is_double_click = if let Some(last_click) = self.last_left_click {
                    now.duration_since(last_click) <= Duration::from_millis(350)
                } else {
                    false
                };
                self.last_left_click = Some(now);

                if is_double_click {
                    if self.state == CapriState::Sleeping {
                        self.wake_up(now, tx_term);
                    } else {
                        self.poke(now);
                    }
                } else if self.state == CapriState::Sleeping {
                    self.wake_up(now, tx_term);
                }
            }
        } else if !left_down {
            self.click_latch = false;
        }

        // Character Dragging (Move)
        if !self.resizing {
            if left_down && !self.dragging && over_capri && !shift_down && !over_corner {
                self.dragging = true;
                self.last_interaction = now;
                if self.state != CapriState::Sleeping {
                    self.state = CapriState::Dangle;
                    self.state_start = now;
                }

                if let Some(ref tracker) = self.tracker {
                    if let Some(cursor) = tracker.get_global_cursor() {
                        self.drag_offset = (cursor.0 - self.win_pos.0, cursor.1 - self.win_pos.1);
                    }
                }
            }

            if left_down && self.dragging {
                self.last_interaction = now;
                if let Some(ref tracker) = self.tracker {
                    if let Some(cursor) = tracker.get_global_cursor() {
                        let target_x = cursor.0 - self.drag_offset.0;
                        let target_y = cursor.1 - self.drag_offset.1;

                        if target_x != self.win_pos.0 || target_y != self.win_pos.1 {
                            self.win_pos = (target_x, target_y);
                            self.window.set_position(target_x, target_y);
                        }
                    }
                }
            } else if !left_down && self.dragging {
                self.dragging = false;
                if self.state != CapriState::Sleeping {
                    self.state = if self.is_perched { CapriState::Perch } else { CapriState::Idle };
                    self.state_start = now;
                }
            }
        }
    }

    pub fn set_size(&mut self, new_w: u32, new_h: u32) {
        let new_w = new_w.clamp(180, 1600);
        let new_h = new_h.clamp(140, 1200);
        if (new_w, new_h) != self.win_size {
            self.win_size = (new_w, new_h);
            if let Some(ref tracker) = self.tracker {
                let handle = self.window.get_window_handle();
                tracker.resize_window(handle, new_w, new_h);
            }
        }
    }

    pub fn adjust_size(&mut self, factor: f32) {
        let cur_size = self.win_size.0 as f32;
        let new_w = ((cur_size * factor).round() as u32).clamp(180, 1600);
        let new_h = ((new_w as f32 * (BUF_HEIGHT as f32 / BUF_WIDTH as f32)).round() as u32).clamp(140, 1200);
        self.set_size(new_w, new_h);
    }

    fn update_state(&mut self, now: Instant, tx_term: &Sender<TerminalNotification>) {
        if self.dragging {
            return;
        }

        // 60-second inactivity sleep timeout matching blueprint
        if self.state != CapriState::Sleeping && self.state != CapriState::Wake {
            if now.duration_since(self.last_interaction) >= Duration::from_secs(60) {
                self.go_to_sleep(now, tx_term);
                return;
            }
        }

        // Check if talking duration finished
        if let Some(until) = self.talking_until {
            if now >= until {
                self.talking_until = None;
                self.state = if self.is_perched { CapriState::Perch } else { CapriState::Idle };
                self.state_start = now;
            }
            return;
        }

        let elapsed = now.duration_since(self.state_start).as_secs_f64();

        match self.state {
            CapriState::Idle => {
                if now >= self.next_blink {
                    self.state = CapriState::Blink;
                    self.state_start = now;
                    // 4000ms ± 800ms jitter matching blueprint
                    self.next_blink = now + Duration::from_millis(fastrand::u64(3200..4800));
                } else if now >= self.next_quirk {
                    self.state = if fastrand::bool() { CapriState::Glitch } else { CapriState::Grin };
                    self.state_start = now;
                    self.next_quirk = now + Duration::from_secs(fastrand::u64(8..16));
                }
            }
            CapriState::Blink => {
                if elapsed >= 0.15 { // 150ms hold matching blueprint
                    self.state = CapriState::Idle;
                    self.state_start = now;
                }
            }
            CapriState::Glitch => {
                if elapsed >= (10.0 / 15.0) {
                    self.state = CapriState::Idle;
                    self.state_start = now;
                }
            }
            CapriState::Grin => {
                if elapsed >= 1.20 { // 1200ms hold matching blueprint
                    self.state = CapriState::Idle;
                    self.state_start = now;
                }
            }
            CapriState::Wake => {
                let wake_dur = if self.theme == CapriTheme::Hd { 1.20 } else { 13.0 / 15.0 };
                if elapsed >= wake_dur {
                    self.state = CapriState::Idle;
                    self.state_start = now;
                }
            }
            CapriState::Perch => {
                if now >= self.next_blink {
                    self.state = CapriState::PerchBlink;
                    self.state_start = now;
                    self.next_blink = now + Duration::from_millis(fastrand::u64(3500..5500));
                } else if now >= self.next_quirk {
                    self.state = CapriState::PerchGlitch;
                    self.state_start = now;
                    self.next_quirk = now + Duration::from_secs(fastrand::u64(10..20));
                }
            }
            CapriState::PerchBlink => {
                if elapsed >= 0.15 {
                    self.state = CapriState::Perch;
                    self.state_start = now;
                }
            }
            CapriState::PerchGlitch => {
                if elapsed >= (10.0 / 15.0) {
                    self.state = CapriState::Perch;
                    self.state_start = now;
                }
            }
            CapriState::Sleeping => {
                // Sleep indefinitely until woken
            }
            _ => {}
        }
    }

    fn render(&mut self, now: Instant) {
        Renderer::clear(&mut self.buffer);

        let elapsed = now.duration_since(self.state_start).as_secs_f64();

        let (clip, dest_x, dest_y): (&Clip, i32, i32) = match self.theme {
            CapriTheme::Hd => {
                let c = match self.state {
                    CapriState::Idle => &self.sprites.hd_idle,
                    CapriState::Blink => &self.sprites.hd_blink,
                    CapriState::Glitch => &self.sprites.hd_blink,
                    CapriState::Grin => &self.sprites.hd_happy,
                    CapriState::Dangle => &self.sprites.hd_happy,
                    CapriState::Talking => {
                        // Alternate between idle and happy expression while talking
                        if (elapsed * 4.0) as usize % 2 == 0 {
                            &self.sprites.hd_happy
                        } else {
                            &self.sprites.hd_idle
                        }
                    }
                    CapriState::Sleeping => &self.sprites.hd_sleeping,
                    CapriState::Wake => &self.sprites.hd_happy,
                    CapriState::Perch | CapriState::PerchBlink | CapriState::PerchGlitch => &self.sprites.hd_idle,
                };
                (c, 0, 0)
            }
            CapriTheme::Pixel => {
                let dx = if self.is_perched { 64 } else { 136 };
                let c = match self.state {
                    CapriState::Idle => &self.sprites.idle,
                    CapriState::Blink => &self.sprites.blink,
                    CapriState::Glitch => &self.sprites.glitch,
                    CapriState::Grin => &self.sprites.grin,
                    CapriState::Dangle => &self.sprites.dangle,
                    CapriState::Talking => {
                        if self.is_perched {
                            &self.sprites.perch_glitch
                        } else {
                            &self.sprites.mouth
                        }
                    }
                    CapriState::Sleeping => &self.sprites.sleep,
                    CapriState::Wake => &self.sprites.wake,
                    CapriState::Perch => &self.sprites.perch,
                    CapriState::PerchBlink => &self.sprites.perch_blink,
                    CapriState::PerchGlitch => &self.sprites.perch_glitch,
                };
                (c, dx, 0)
            }
        };

        let frame_pixels = clip.get_frame(elapsed);

        Renderer::blit_sprite(
            &mut self.buffer,
            BUF_WIDTH,
            BUF_HEIGHT,
            frame_pixels,
            clip.width,
            clip.height,
            dest_x,
            dest_y,
        );

        // Draw subtle cybernetic resize grip in bottom-right corner
        Renderer::draw_corner_grip(&mut self.buffer, BUF_WIDTH, BUF_HEIGHT);
    }
}
