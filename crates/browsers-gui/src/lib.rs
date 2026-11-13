use browsers_browser::Browser;
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::EventLoop,
    window::{Window, WindowAttributes, WindowId},
};

pub struct GuiBrowser {
    #[allow(dead_code)]
    window: Option<Arc<Window>>,
    browser: Browser,
    width: u32,
    height: u32,
}

impl GuiBrowser {
    pub fn new(width: u32, height: u32) -> Self {
        GuiBrowser {
            window: None,
            browser: Browser::new(),
            width,
            height,
        }
    }

    pub fn load_html(&mut self, html: &str) {
        self.browser.load_html(html);
        self.browser.set_viewport(self.width, self.height);
    }

    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        let mut app = GuiBrowserApp {
            window: None,
            browser: Some(self.browser),
            width: self.width,
            height: self.height,
            context: None,
            surface: None,
            needs_redraw: true,
        };
        event_loop.run_app(&mut app)?;
        Ok(())
    }
}

struct GuiBrowserApp {
    window: Option<Arc<Window>>,
    browser: Option<Browser>,
    width: u32,
    height: u32,
    context: Option<softbuffer::Context<Arc<Window>>>,
    surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
    needs_redraw: bool,
}

impl ApplicationHandler for GuiBrowserApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = WindowAttributes::default()
                .with_title("Browser Runtime")
                .with_inner_size(LogicalSize::new(self.width, self.height));

            let window = Arc::new(event_loop.create_window(attrs).unwrap());
            let context = softbuffer::Context::new(window.clone()).unwrap();
            let surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

            self.window = Some(window);
            self.context = Some(context);
            self.surface = Some(surface);

            if let Some(ref mut browser) = self.browser {
                browser.build_layout();
                browser.build_display_list();
                self.needs_redraw = true;
            }

            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let (Some(surface), Some(browser)) = (&mut self.surface, &self.browser) {
                    let img = browser.render();
                    let (width, height) = surface.window().inner_size().into();

                    surface
                        .resize(
                            NonZeroU32::new(width).unwrap(),
                            NonZeroU32::new(height).unwrap(),
                        )
                        .unwrap();

                    let mut buffer = surface.buffer_mut().unwrap();
                    for y in 0..height.min(img.height()) {
                        for x in 0..width.min(img.width()) {
                            let pixel = img.get_pixel(x, y);
                            // softbuffer macOS: NoneSkipFirst + Order32Little → u32 = 0xXXRRGGBB
                            let color = ((pixel[3] as u32) << 24)
                                | ((pixel[0] as u32) << 16)
                                | ((pixel[1] as u32) << 8)
                                | (pixel[2] as u32);
                            buffer[(y * width + x) as usize] = color;
                        }
                    }
                    buffer.present().unwrap();
                }
                self.needs_redraw = false;
            }
            WindowEvent::Resized(size) => {
                self.width = size.width;
                self.height = size.height;
                self.needs_redraw = true;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_gui_browser() {
        let gui = GuiBrowser::new(800, 600);
        assert_eq!(gui.width, 800);
        assert_eq!(gui.height, 600);
    }

    #[test]
    fn load_html_into_gui() {
        let mut gui = GuiBrowser::new(800, 600);
        gui.load_html("<html><body><div>Hello</div></body></html>");
        assert!(gui.browser.dom().is_some());
    }
}
