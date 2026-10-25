#![allow(dead_code)]
use fontdue::{Font, FontSettings};

pub const COLOR_PANEL: u32 = 0xEE090B0F;
pub const COLOR_BORDER: u32 = 0xFF3AD6EB;
pub const COLOR_YELLOW: u32 = 0xFFF4D646;
pub const COLOR_TEXT: u32 = 0xFFE8ECF0;
pub const COLOR_DIM: u32 = 0xFF7A808A;
#[allow(dead_code)]
pub const COLOR_RED: u32 = 0xFFE82440;

pub struct Renderer {
    pub font: Font,
}

impl Renderer {
    pub fn new() -> Result<Self, String> {
        let font_bytes = include_bytes!("../assets/fonts/ShareTechMono-Regular.ttf");
        let font = Font::from_bytes(font_bytes as &[u8], FontSettings::default())
            .map_err(|e| format!("Failed to parse font: {}", e))?;
        Ok(Renderer { font })
    }

    pub fn clear(buffer: &mut [u32]) {
        for pixel in buffer.iter_mut() {
            *pixel = 0x00000000; // Fully transparent
        }
    }

    /// Draw a subtle cyberpunk corner grip at the bottom-right
    pub fn draw_corner_grip(buffer: &mut [u32], buf_width: usize, buf_height: usize) {
        let color = 0x663AD6EB; // Subtle translucent cyan
        for i in 0..4 {
            let offset = i * 4;
            let x0 = buf_width as i32 - 14 - offset;
            let y0 = buf_height as i32 - 4;
            let x1 = buf_width as i32 - 4;
            let y1 = buf_height as i32 - 14 - offset;

            let steps = 10 + offset;
            for s in 0..=steps {
                let px = x0 + ((x1 - x0) * s) / steps;
                let py = y0 + ((y1 - y0) * s) / steps;
                if px >= 0 && px < buf_width as i32 && py >= 0 && py < buf_height as i32 {
                    let idx = (py as usize) * buf_width + (px as usize);
                    let dst = buffer[idx];
                    let dst_a = (dst >> 24) & 0xFF;
                    if dst_a == 0 {
                        buffer[idx] = color;
                    }
                }
            }
        }
    }

    /// Blit an ARGB sprite frame onto the window buffer with alpha blending
    pub fn blit_sprite(
        buffer: &mut [u32],
        buf_width: usize,
        buf_height: usize,
        sprite_pixels: &[u32],
        sp_width: usize,
        sp_height: usize,
        dest_x: i32,
        dest_y: i32,
    ) {
        for sy in 0..sp_height {
            let dy = dest_y + sy as i32;
            if dy < 0 || dy >= buf_height as i32 {
                continue;
            }
            for sx in 0..sp_width {
                let dx = dest_x + sx as i32;
                if dx < 0 || dx >= buf_width as i32 {
                    continue;
                }

                let src = sprite_pixels[sy * sp_width + sx];
                let src_a = (src >> 24) & 0xFF;
                if src_a == 0 {
                    continue;
                }

                let d_idx = (dy as usize) * buf_width + (dx as usize);
                if src_a >= 250 {
                    buffer[d_idx] = src;
                } else {
                    let dst = buffer[d_idx];
                    let dst_a = (dst >> 24) & 0xFF;
                    let dst_r = (dst >> 16) & 0xFF;
                    let dst_g = (dst >> 8) & 0xFF;
                    let dst_b = dst & 0xFF;

                    let src_r = (src >> 16) & 0xFF;
                    let src_g = (src >> 8) & 0xFF;
                    let src_b = src & 0xFF;

                    let inv_a = 255 - src_a;
                    let out_r = (src_r * src_a + dst_r * inv_a) / 255;
                    let out_g = (src_g * src_a + dst_g * inv_a) / 255;
                    let out_b = (src_b * src_a + dst_b * inv_a) / 255;
                    let out_a = src_a.max(dst_a);

                    buffer[d_idx] = (out_a << 24) | (out_r << 16) | (out_g << 8) | out_b;
                }
            }
        }
    }

    /// Fill a rectangle with an ARGB color
    pub fn fill_rect(
        buffer: &mut [u32],
        buf_width: usize,
        buf_height: usize,
        rx: i32,
        ry: i32,
        rw: i32,
        rh: i32,
        color: u32,
    ) {
        let col_a = (color >> 24) & 0xFF;
        if col_a == 0 {
            return;
        }

        let x_start = rx.max(0) as usize;
        let x_end = (rx + rw).min(buf_width as i32).max(0) as usize;
        let y_start = ry.max(0) as usize;
        let y_end = (ry + rh).min(buf_height as i32).max(0) as usize;

        for y in y_start..y_end {
            for x in x_start..x_end {
                let idx = y * buf_width + x;
                if col_a == 255 {
                    buffer[idx] = color;
                } else {
                    let dst = buffer[idx];
                    let dst_a = (dst >> 24) & 0xFF;
                    let dst_r = (dst >> 16) & 0xFF;
                    let dst_g = (dst >> 8) & 0xFF;
                    let dst_b = dst & 0xFF;

                    let src_r = (color >> 16) & 0xFF;
                    let src_g = (color >> 8) & 0xFF;
                    let src_b = color & 0xFF;

                    let inv_a = 255 - col_a;
                    let out_r = (src_r * col_a + dst_r * inv_a) / 255;
                    let out_g = (src_g * col_a + dst_g * inv_a) / 255;
                    let out_b = (src_b * col_a + dst_b * inv_a) / 255;
                    let out_a = col_a.max(dst_a);

                    buffer[idx] = (out_a << 24) | (out_r << 16) | (out_g << 8) | out_b;
                }
            }
        }
    }

    /// Draw a 1px border rectangle
    pub fn stroke_rect(
        buffer: &mut [u32],
        buf_width: usize,
        buf_height: usize,
        rx: i32,
        ry: i32,
        rw: i32,
        rh: i32,
        color: u32,
    ) {
        Self::fill_rect(buffer, buf_width, buf_height, rx, ry, rw, 1, color);
        Self::fill_rect(buffer, buf_width, buf_height, rx, ry + rh - 1, rw, 1, color);
        Self::fill_rect(buffer, buf_width, buf_height, rx, ry, 1, rh, color);
        Self::fill_rect(buffer, buf_width, buf_height, rx + rw - 1, ry, 1, rh, color);
    }

    /// Draw text string with fontdue
    pub fn draw_text(
        &self,
        buffer: &mut [u32],
        buf_width: usize,
        buf_height: usize,
        text: &str,
        start_x: i32,
        start_y: i32,
        font_size: f32,
        color: u32,
    ) -> i32 {
        let mut cur_x = start_x;
        let base_r = (color >> 16) & 0xFF;
        let base_g = (color >> 8) & 0xFF;
        let base_b = color & 0xFF;

        for ch in text.chars() {
            if ch == ' ' {
                cur_x += (font_size * 0.55) as i32;
                continue;
            }
            if ch == '\t' {
                cur_x += (font_size * 1.5) as i32;
                continue;
            }

            let (metrics, bitmap) = self.font.rasterize(ch, font_size);
            let gx = cur_x + metrics.xmin;
            let gy = start_y + (font_size as i32 - metrics.height as i32 - metrics.ymin);

            for row in 0..metrics.height {
                let py = gy + row as i32;
                if py < 0 || py >= buf_height as i32 {
                    continue;
                }
                for col in 0..metrics.width {
                    let px = gx + col as i32;
                    if px < 0 || px >= buf_width as i32 {
                        continue;
                    }

                    let alpha = bitmap[row * metrics.width + col] as u32;
                    if alpha == 0 {
                        continue;
                    }

                    let idx = (py as usize) * buf_width + (px as usize);
                    let dst = buffer[idx];
                    let dst_a = (dst >> 24) & 0xFF;
                    let dst_r = (dst >> 16) & 0xFF;
                    let dst_g = (dst >> 8) & 0xFF;
                    let dst_b = dst & 0xFF;

                    let inv_a = 255 - alpha;
                    let out_r = (base_r * alpha + dst_r * inv_a) / 255;
                    let out_g = (base_g * alpha + dst_g * inv_a) / 255;
                    let out_b = (base_b * alpha + dst_b * inv_a) / 255;
                    let out_a = alpha.max(dst_a);

                    buffer[idx] = (out_a << 24) | (out_r << 16) | (out_g << 8) | out_b;
                }
            }
            cur_x += metrics.advance_width as i32;
        }

        cur_x
    }

    /// Render a wrapped cyber dialogue speech bubble with an interactive terminal input prompt
    pub fn draw_bubble(
        &self,
        buffer: &mut [u32],
        buf_width: usize,
        buf_height: usize,
        box_x: i32,
        box_y: i32,
        box_w: i32,
        box_h: i32,
        title: &str,
        lines: &[String],
        input_info: Option<(&str, usize, bool)>, // (text, cursor_pos, show_cursor)
    ) {
        // Background plate
        Self::fill_rect(buffer, buf_width, buf_height, box_x, box_y, box_w, box_h, COLOR_PANEL);
        // Cyber border
        Self::stroke_rect(buffer, buf_width, buf_height, box_x, box_y, box_w, box_h, COLOR_BORDER);

        // Cyber cut corner accent
        Self::fill_rect(buffer, buf_width, buf_height, box_x, box_y, 16, 2, COLOR_YELLOW);
        Self::fill_rect(buffer, buf_width, buf_height, box_x, box_y, 2, 16, COLOR_YELLOW);

        // Title
        self.draw_text(buffer, buf_width, buf_height, title, box_x + 10, box_y + 8, 13.0, COLOR_YELLOW);

        // Content lines
        let mut cur_y = box_y + 26;
        for line in lines {
            if cur_y + 14 > box_y + box_h - 26 {
                break;
            }
            self.draw_text(buffer, buf_width, buf_height, line, box_x + 10, cur_y, 12.0, COLOR_TEXT);
            cur_y += 15;
        }

        // Editable Terminal Input Box
        if let Some((input, cursor_pos, show_cursor)) = input_info {
            let input_y = box_y + box_h - 22;
            let input_h = 20;
            let input_x = box_x + 8;
            let input_w = box_w - 16;

            Self::fill_rect(buffer, buf_width, buf_height, input_x, input_y - 2, input_w, input_h, 0xEE11151C);
            Self::stroke_rect(buffer, buf_width, buf_height, input_x, input_y - 2, input_w, input_h, COLOR_DIM);

            let prompt_x = self.draw_text(buffer, buf_width, buf_height, "> ", input_x + 6, input_y + 1, 12.0, COLOR_YELLOW);

            let chars: Vec<char> = input.chars().collect();
            let total_chars = chars.len();
            let cursor = cursor_pos.min(total_chars);

            let max_visible_chars = ((input_w - 28) as f32 / 7.2) as usize;
            let (start_idx, end_idx) = if total_chars <= max_visible_chars {
                (0, total_chars)
            } else if cursor < max_visible_chars {
                (0, max_visible_chars)
            } else {
                let start = cursor.saturating_sub(max_visible_chars - 3);
                let end = (start + max_visible_chars).min(total_chars);
                (start, end)
            };

            let visible_slice: String = chars[start_idx..end_idx].iter().collect();
            let cursor_rel = cursor.saturating_sub(start_idx);

            let mut cur_x = prompt_x;
            let mut cursor_pixel_x = cur_x;

            for (idx, ch) in visible_slice.chars().enumerate() {
                if idx == cursor_rel {
                    cursor_pixel_x = cur_x;
                }
                cur_x = self.draw_text(buffer, buf_width, buf_height, &ch.to_string(), cur_x, input_y + 1, 12.0, COLOR_BORDER);
            }
            if cursor_rel == visible_slice.chars().count() {
                cursor_pixel_x = cur_x;
            }

            // Blinking block cursor
            if show_cursor {
                let cursor_w = 7;
                let cursor_h = 13;
                Self::fill_rect(buffer, buf_width, buf_height, cursor_pixel_x, input_y + 2, cursor_w, cursor_h, 0x883AD6EB);
            }
        }
    }
}
