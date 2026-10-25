use image::GenericImageView;

pub struct Clip {
    pub name: &'static str,
    pub count: usize,
    pub fps: f64,
    pub width: usize,
    pub height: usize,
    pub frames: Vec<Vec<u32>>, // ARGB formatted pixels for each frame
}

impl Clip {
    pub fn from_png_bytes(name: &'static str, bytes: &[u8], count: usize, fps: f64) -> Result<Self, String> {
        let img = image::load_from_memory(bytes)
            .map_err(|e| format!("Failed to decode sprite {}: {}", name, e))?;

        let (total_width, height) = img.dimensions();
        let frame_width = total_width / count as u32;

        let mut frames = Vec::with_capacity(count);

        for f in 0..count {
            let mut frame_pixels = Vec::with_capacity((frame_width * height) as usize);
            let offset_x = (f as u32) * frame_width;

            for y in 0..height {
                for x in 0..frame_width {
                    let pixel = img.get_pixel(offset_x + x, y);
                    let r = pixel[0] as u32;
                    let g = pixel[1] as u32;
                    let b = pixel[2] as u32;
                    let a = pixel[3] as u32;
                    // minifb format: 0xAARRGGBB
                    let argb = (a << 24) | (r << 16) | (g << 8) | b;
                    frame_pixels.push(argb);
                }
            }
            frames.push(frame_pixels);
        }

        Ok(Clip {
            name,
            count,
            fps,
            width: frame_width as usize,
            height: height as usize,
            frames,
        })
    }

    pub fn get_frame(&self, time_sec: f64) -> &Vec<u32> {
        if self.count == 0 {
            panic!("Clip {} has 0 frames", self.name);
        }
        let total_duration = (self.count as f64) / self.fps;
        let t = time_sec % total_duration;
        let index = ((t * self.fps) as usize).min(self.count - 1);
        &self.frames[index]
    }
}

#[allow(dead_code)]
pub struct SpriteCollection {
    pub idle: Clip,
    pub blink: Clip,
    pub blink_glitch: Clip,
    pub grin: Clip,
    pub glitch: Clip,
    pub dangle: Clip,
    pub sleep: Clip,
    pub wake: Clip,
    pub mouth: Clip,
    pub perch: Clip,
    pub perch_blink: Clip,
    pub perch_glitch: Clip,
}

impl SpriteCollection {
    pub fn load_embedded() -> Result<Self, String> {
        let idle = Clip::from_png_bytes(
            "idle",
            include_bytes!("../assets/sprites/idle.png"),
            16,
            8.0,
        )?;
        let blink = Clip::from_png_bytes(
            "blink",
            include_bytes!("../assets/sprites/blink.png"),
            4,
            8.0,
        )?;
        let blink_glitch = Clip::from_png_bytes(
            "blink_glitch",
            include_bytes!("../assets/sprites/blink_glitch.png"),
            4,
            8.0,
        )?;
        let grin = Clip::from_png_bytes(
            "grin",
            include_bytes!("../assets/sprites/grin.png"),
            16,
            8.0,
        )?;
        let glitch = Clip::from_png_bytes(
            "glitch",
            include_bytes!("../assets/sprites/glitch.png"),
            10,
            15.0,
        )?;
        let dangle = Clip::from_png_bytes(
            "dangle",
            include_bytes!("../assets/sprites/dangle.png"),
            24,
            10.0,
        )?;
        let sleep = Clip::from_png_bytes(
            "sleep",
            include_bytes!("../assets/sprites/sleep.png"),
            16,
            6.0,
        )?;
        let wake = Clip::from_png_bytes(
            "wake",
            include_bytes!("../assets/sprites/wake.png"),
            13,
            15.0,
        )?;
        let mouth = Clip::from_png_bytes(
            "mouth",
            include_bytes!("../assets/sprites/mouth.png"),
            5,
            12.0,
        )?;
        let perch = Clip::from_png_bytes(
            "perch",
            include_bytes!("../assets/sprites/perch.png"),
            16,
            8.0,
        )?;
        let perch_blink = Clip::from_png_bytes(
            "perch_blink",
            include_bytes!("../assets/sprites/perch_blink.png"),
            4,
            8.0,
        )?;
        let perch_glitch = Clip::from_png_bytes(
            "perch_glitch",
            include_bytes!("../assets/sprites/perch_glitch.png"),
            10,
            15.0,
        )?;

        Ok(SpriteCollection {
            idle,
            blink,
            blink_glitch,
            grin,
            glitch,
            dangle,
            sleep,
            wake,
            mouth,
            perch,
            perch_blink,
            perch_glitch,
        })
    }
}
