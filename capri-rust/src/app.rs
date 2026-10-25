use minifb::{Key, MouseButton, MouseMode, ScaleMode, Window, WindowOptions};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

use crate::render::Renderer;
use crate::sprites::{Clip, SpriteCollection};
use crate::terminal::{MascotCommand, TerminalNotification};
use crate::tracker::X11CursorTracker;

pub const BUF_WIDTH: usize = 384;
pub const BUF_HEIGHT: usize = 384;

#[derive(PartialEq)]
pub enum CapriState {
    Idle,
    Blink,
    Glitch,
    Grin,
    Dangle,
    Talking,
    Perch,
    PerchBlink,
    PerchGlitch,
}

pub struct App {
    pub window: Window,
    pub buffer: Vec<u32>,
    pub sprites: SpriteCollection,
    pub tracker: Option<X11CursorTracker>,
    pub state: CapriState,
    pub state_start: Instant,
    pub next_blink: Instant,
    pub next_quirk: Instant,
    pub talking_until: Option<Instant>,
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
    pub fn new(scale: f32) -> Result<Self, String> {
        let initial_w = ((BUF_WIDTH as f32) * scale).round() as usize;
        let initial_h = ((BUF_HEIGHT as f32) * scale).round() as usize;

        let opts = WindowOptions {
            borderless: true,
            title: false,
            resize: true,
            scale_mode: ScaleMode::AspectRatioStretch,
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
            state: CapriState::Idle,
            state_start: now,
            next_blink: now + Duration::from_secs(3),
            next_quirk: now + Duration::from_secs(8),
            talking_until: None,
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

            // 1. Check messages from Terminal REPL
            while let Ok(cmd) = rx_cmd.try_recv() {
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
                    MascotCommand::Exit => {
                        return Ok(());
                    }
                }
            }

            self.handle_input(now);
            self.update_state(now);
            self.render(now);

            self.window
                .update_with_buffer(&self.buffer, BUF_WIDTH, BUF_HEIGHT)
                .map_err(|e| format!("Failed to update window: {}", e))?;
        }

        let _ = tx_term.send(TerminalNotification::MascotClosed);
        Ok(())
    }

    fn handle_input(&mut self, now: Instant) {
        // 1. Sync win_size with actual window dimensions (e.g. if resized by KDE Alt+Right-Click)
        let (actual_w, actual_h) = self.window.get_size();
        if actual_w > 0 && actual_h > 0 {
            if actual_w as u32 != self.win_size.0 || actual_h as u32 != self.win_size.1 {
                self.win_size = (actual_w as u32, actual_h as u32);
            }
        }

        // 2. Normalize mouse position to 384x384 buffer coordinates
        let scale_factor = ((self.win_size.0 as f32) / (BUF_WIDTH as f32))
            .min((self.win_size.1 as f32) / (BUF_HEIGHT as f32))
            .max(0.1);

        let mouse_pos = self.window.get_mouse_pos(MouseMode::Pass);
        let mouse_buf = mouse_pos.map(|(mx, my)| (mx / scale_factor, my / scale_factor));

        // Toggle Perch with Right Mouse Button (click event)
        let right_pressed = self.window.get_mouse_down(MouseButton::Right);
        if right_pressed && !self.right_down && !self.dragging && !self.resizing {
            self.right_down = true;
            self.is_perched = !self.is_perched;
            self.state = if self.is_perched { CapriState::Perch } else { CapriState::Idle };
            self.state_start = now;
        } else if !right_pressed {
            self.right_down = false;
        }

        // Toggle Perch with 'P' key
        if self.window.is_key_pressed(Key::P, minifb::KeyRepeat::No) {
            self.is_perched = !self.is_perched;
            self.state = if self.is_perched { CapriState::Perch } else { CapriState::Idle };
            self.state_start = now;
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

        // Check if cursor is over corner resize grip in buffer space (bottom-right 40x40 area)
        let over_corner = if let Some((bx, by)) = mouse_buf {
            bx >= 330.0 && by >= 330.0
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
                        let new_w = ((self.resize_start_size.0 as isize + delta).clamp(160, 1600)) as u32;
                        let new_h = new_w; // Maintain 1:1 aspect ratio
                        self.set_size(new_w, new_h);
                    }
                }
            } else {
                self.resizing = false;
            }
        }

        // Character Dragging (Move)
        if !self.resizing {
            if let Some((bx, by)) = mouse_buf {
                let sprite_x = if self.is_perched { 0 } else { 72 };
                let sprite_w = if self.is_perched { 384 } else { 240 };
                let sprite_h = 384;

                let over_capri = bx >= sprite_x as f32
                    && bx <= (sprite_x + sprite_w) as f32
                    && by >= 0.0
                    && by <= sprite_h as f32;

                if left_down && !self.dragging && over_capri && !shift_down && !over_corner {
                    self.dragging = true;
                    self.state = CapriState::Dangle;
                    self.state_start = now;

                    if let Some(ref tracker) = self.tracker {
                        if let Some(cursor) = tracker.get_global_cursor() {
                            self.drag_offset = (cursor.0 - self.win_pos.0, cursor.1 - self.win_pos.1);
                        }
                    }
                }
            }

            if left_down && self.dragging {
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
                self.state = if self.is_perched { CapriState::Perch } else { CapriState::Idle };
                self.state_start = now;
            }
        }
    }

    pub fn set_size(&mut self, new_w: u32, new_h: u32) {
        let new_w = new_w.clamp(160, 1600);
        let new_h = new_h.clamp(160, 1600);
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
        let new_size = ((cur_size * factor).round() as u32).clamp(160, 1600);
        self.set_size(new_size, new_size);
    }

    fn update_state(&mut self, now: Instant) {
        if self.dragging {
            return;
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
                    self.next_blink = now + Duration::from_secs(fastrand::u64(3..7));
                } else if now >= self.next_quirk {
                    self.state = if fastrand::bool() { CapriState::Glitch } else { CapriState::Grin };
                    self.state_start = now;
                    self.next_quirk = now + Duration::from_secs(fastrand::u64(8..16));
                }
            }
            CapriState::Blink => {
                if elapsed >= (4.0 / 8.0) {
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
                if elapsed >= (16.0 / 8.0) {
                    self.state = CapriState::Idle;
                    self.state_start = now;
                }
            }
            CapriState::Perch => {
                if now >= self.next_blink {
                    self.state = CapriState::PerchBlink;
                    self.state_start = now;
                    self.next_blink = now + Duration::from_secs(fastrand::u64(4..8));
                } else if now >= self.next_quirk {
                    self.state = CapriState::PerchGlitch;
                    self.state_start = now;
                    self.next_quirk = now + Duration::from_secs(fastrand::u64(10..20));
                }
            }
            CapriState::PerchBlink => {
                if elapsed >= (4.0 / 8.0) {
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
            _ => {}
        }
    }

    fn render(&mut self, now: Instant) {
        Renderer::clear(&mut self.buffer);

        let elapsed = now.duration_since(self.state_start).as_secs_f64();

        let clip: &Clip = match self.state {
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
            CapriState::Perch => &self.sprites.perch,
            CapriState::PerchBlink => &self.sprites.perch_blink,
            CapriState::PerchGlitch => &self.sprites.perch_glitch,
        };

        let frame_pixels = clip.get_frame(elapsed);

        let dest_x = if self.is_perched { 0 } else { 72 };
        let dest_y = 0;

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
