use x11_dl::xlib::Xlib;

pub struct X11CursorTracker {
    xlib: Xlib,
    display: *mut x11_dl::xlib::Display,
    root: x11_dl::xlib::Window,
}

// Mark Send since we own display and access from main thread
unsafe impl Send for X11CursorTracker {}

impl X11CursorTracker {
    pub fn new() -> Option<Self> {
        let xlib = Xlib::open().ok()?;
        unsafe {
            let display = (xlib.XOpenDisplay)(std::ptr::null());
            if display.is_null() {
                return None;
            }
            let root = (xlib.XDefaultRootWindow)(display);
            Some(Self { xlib, display, root })
        }
    }

    pub fn get_global_cursor(&self) -> Option<(isize, isize)> {
        unsafe {
            let mut root_return = 0;
            let mut child_return = 0;
            let mut root_x = 0;
            let mut root_y = 0;
            let mut win_x = 0;
            let mut win_y = 0;
            let mut mask = 0;

            let status = (self.xlib.XQueryPointer)(
                self.display,
                self.root,
                &mut root_return,
                &mut child_return,
                &mut root_x,
                &mut root_y,
                &mut win_x,
                &mut win_y,
                &mut mask,
            );

            if status != 0 {
                Some((root_x as isize, root_y as isize))
            } else {
                None
            }
        }
    }

    pub fn resize_window(&self, window_handle: *mut std::ffi::c_void, width: u32, height: u32) {
        unsafe {
            if !self.display.is_null() && !window_handle.is_null() {
                let win = window_handle as usize as x11_dl::xlib::Window;
                (self.xlib.XResizeWindow)(self.display, win, width, height);
                (self.xlib.XFlush)(self.display);
            }
        }
    }
}

impl Drop for X11CursorTracker {
    fn drop(&mut self) {
        unsafe {
            if !self.display.is_null() {
                (self.xlib.XCloseDisplay)(self.display);
            }
        }
    }
}
