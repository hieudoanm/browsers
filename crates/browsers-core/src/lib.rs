use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use euclid::{Point2D, Scale, Size2D};
use servo::{
    DeviceIntRect, JSValue, JavaScriptEvaluationError, LoadStatus, Preferences, RenderingContext,
    Servo, ServoBuilder, SoftwareRenderingContext, WebView, WebViewBuilder, WebViewDelegate,
};
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum BrowserError {
    #[error("servo initialization failed: {0}")]
    ServoInit(String),

    #[error("navigation failed: {0}")]
    Navigation(String),

    #[error("rendering failed: {0}")]
    Rendering(String),

    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("timeout: {0}")]
    Timeout(String),
}

pub struct BrowserConfig {
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub device_scale_factor: f32,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            viewport_width: 1280,
            viewport_height: 720,
            device_scale_factor: 1.0,
        }
    }
}

pub struct BrowserContext {
    servo: Servo,
    rendering_context: Rc<dyn RenderingContext>,
    config: BrowserConfig,
}

impl BrowserContext {
    pub fn new(config: BrowserConfig) -> Result<Self, BrowserError> {
        let size = dpi::PhysicalSize::new(config.viewport_width, config.viewport_height);

        let rendering_context: Rc<dyn RenderingContext> = Rc::new(
            SoftwareRenderingContext::new(size)
                .map_err(|e| BrowserError::Rendering(format!("{e:?}")))?,
        );

        let _ = rendering_context.make_current();

        let prefs = Preferences {
            network_http_proxy_uri: String::new(),
            network_https_proxy_uri: String::new(),
            ..Default::default()
        };

        let servo: Servo = ServoBuilder::default().preferences(prefs).build();
        servo.setup_logging();

        Ok(Self {
            servo,
            rendering_context,
            config,
        })
    }

    pub fn new_page(&self, url: &str) -> Result<Page, BrowserError> {
        let parsed = Url::parse(url).map_err(|e| BrowserError::InvalidUrl(e.to_string()))?;

        let delegate = Rc::new(PageDelegate::new());
        let handle = delegate.handle();

        let webview: WebView = WebViewBuilder::new(&self.servo, self.rendering_context.clone())
            .url(parsed)
            .hidpi_scale_factor(Scale::new(self.config.device_scale_factor))
            .delegate(delegate as Rc<dyn WebViewDelegate>)
            .build();

        webview.resize(dpi::PhysicalSize::new(
            self.config.viewport_width,
            self.config.viewport_height,
        ));

        Ok(Page {
            webview,
            handle,
            rendering_context: self.rendering_context.clone(),
            servo: self.servo.clone(),
        })
    }

    pub fn viewport_size(&self) -> (u32, u32) {
        (self.config.viewport_width, self.config.viewport_height)
    }

    pub fn spin_event_loop(&self) {
        self.servo.spin_event_loop();
    }
}

pub struct Page {
    webview: WebView,
    handle: Rc<PageHandle>,
    rendering_context: Rc<dyn RenderingContext>,
    servo: Servo,
}

impl Page {
    pub fn goto(&self, url: &str) -> Result<(), BrowserError> {
        let parsed = Url::parse(url).map_err(|e| BrowserError::InvalidUrl(e.to_string()))?;

        // Allow the browsing context for this WebView to be registered by the
        // constellation before issuing a top-level load, so the load is never
        // dropped as "unknown browsing context".
        self.servo.spin_event_loop();
        self.webview.load(parsed);
        self.servo.spin_event_loop();
        Ok(())
    }

    pub fn wait_for_load(&self, timeout_ms: u64) -> Result<(), BrowserError> {
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);

        while !self.handle.is_loaded() {
            if Instant::now() >= deadline {
                return Err(BrowserError::Timeout("waiting for page load".into()));
            }

            self.servo.spin_event_loop();
            self.rendering_context
                .make_current()
                .map_err(|e| BrowserError::Rendering(format!("{e:?}")))?;
            std::thread::sleep(Duration::from_millis(1));
        }

        Ok(())
    }

    pub fn wait_for_render(&self, timeout_ms: u64) -> Result<(), BrowserError> {
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        let frames_at_start = self.handle.frames();

        while self.handle.frames() <= frames_at_start {
            if Instant::now() >= deadline {
                return Err(BrowserError::Timeout("waiting for render".into()));
            }

            self.servo.spin_event_loop();
            self.rendering_context
                .make_current()
                .map_err(|e| BrowserError::Rendering(format!("{e:?}")))?;
            std::thread::sleep(Duration::from_millis(1));
        }

        Ok(())
    }

    pub fn screenshot(&self) -> Result<image::RgbaImage, BrowserError> {
        // Capture a rendering-context frame first: take_screenshot only fires
        // its callback once the page has no pending frames, so it never
        // completes for continuously-animating pages. Its presence also makes
        // the surface lose the current frame, so read the surface up front.
        let readback = self.poll_surface_readback(Duration::from_secs(5));
        match self.take_screenshot_frame() {
            Ok(img) => Ok(img),
            Err(_) => {
                log::warn!("take_screenshot timed out; using rendering-context readback");
                readback
            }
        }
    }

    fn poll_surface_readback(&self, timeout: Duration) -> Result<image::RgbaImage, BrowserError> {
        let deadline = Instant::now() + timeout;

        loop {
            let frame = self.surface_readback().ok();
            if frame.as_ref().is_some_and(Page::rendered_content) {
                return Ok(frame.unwrap());
            }

            if Instant::now() >= deadline {
                return frame.ok_or_else(|| BrowserError::Rendering("readback failed".into()));
            }

            self.servo.spin_event_loop();
            std::thread::sleep(Duration::from_millis(16));
        }
    }

    fn rendered_content(img: &image::RgbaImage) -> bool {
        let mut non_white = 0u32;
        for (.., px) in img.enumerate_pixels() {
            if px.0 != [255, 255, 255, 255] && px.0 != [0, 0, 0, 0] {
                non_white += 1;
                if non_white >= 16 {
                    return true;
                }
            }
        }
        false
    }

    fn take_screenshot_frame(&self) -> Result<image::RgbaImage, BrowserError> {
        use std::cell::RefCell;
        use std::rc::Rc;

        let result: Rc<RefCell<Option<image::RgbaImage>>> = Rc::new(RefCell::new(None));
        let result_clone = result.clone();

        self.webview
            .take_screenshot(None, move |screenshot_result| match screenshot_result {
                Ok(img) => {
                    log::info!(
                        "take_screenshot callback: got {}x{}",
                        img.width(),
                        img.height()
                    );
                    *result_clone.borrow_mut() = Some(img);
                }
                Err(e) => log::error!("take_screenshot failed: {e:?}"),
            });

        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(img) = result.borrow().clone() {
                return Ok(img);
            }
            if Instant::now() >= deadline {
                return Err(BrowserError::Timeout("waiting for take_screenshot".into()));
            }
            self.servo.spin_event_loop();
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    pub fn save_screenshot(&self, path: &str) -> Result<(), BrowserError> {
        let img = self.screenshot()?;
        img.save(path)
            .map_err(|e| BrowserError::Rendering(e.to_string()))?;
        Ok(())
    }

    pub fn surface_size(&self) -> (u32, u32) {
        let size = self.rendering_context.size2d();
        (size.width, size.height)
    }

    pub fn webview_size(&self) -> (f32, f32) {
        let size = self.webview.size();
        (size.width, size.height)
    }

    pub fn surface_readback(&self) -> Result<image::RgbaImage, BrowserError> {
        self.rendering_context
            .make_current()
            .map_err(|e| BrowserError::Rendering(format!("make_current failed: {e:?}")))?;
        let size = self.rendering_context.size2d();
        let rect = DeviceIntRect::from_origin_and_size(
            Point2D::new(0, 0),
            Size2D::new(size.width as i32, size.height as i32),
        );
        self.rendering_context
            .read_to_image(rect)
            .ok_or_else(|| BrowserError::Rendering("read_to_image returned None".into()))
    }

    pub fn present(&self) {
        self.rendering_context.present();
    }

    pub fn pixel_summary(img: &image::RgbaImage) -> String {
        let mut distinct: Vec<(u8, u8, u8, u8)> = Vec::new();
        let mut samples: Vec<(u32, u32, u8, u8, u8, u8)> = Vec::new();
        for (x, y, px) in img.enumerate_pixels() {
            let rgba = px.0;
            if distinct.len() < 32 {
                let key = (rgba[0], rgba[1], rgba[2], rgba[3]);
                if !distinct.contains(&key) {
                    distinct.push(key);
                }
            }
            if samples.len() < 4 {
                samples.push((x, y, rgba[0], rgba[1], rgba[2], rgba[3]));
            }
        }
        format!(
            "{}x{} distinct_colors={} samples={:?}",
            img.width(),
            img.height(),
            distinct.len(),
            samples
        )
    }

    pub fn evaluate_javascript<F>(&self, script: &str, callback: F)
    where
        F: FnOnce(Result<JSValue, JavaScriptEvaluationError>) + 'static,
    {
        self.webview.evaluate_javascript(script, callback);
    }

    pub fn dump_html(&self, timeout: Duration) -> Result<String, BrowserError> {
        self.evaluate_string("document.documentElement.outerHTML", timeout)
    }

    pub fn evaluate_string(&self, script: &str, timeout: Duration) -> Result<String, BrowserError> {
        use std::cell::RefCell;

        let result: Rc<RefCell<Option<Result<String, String>>>> = Rc::new(RefCell::new(None));
        let result_clone = result.clone();

        self.webview.evaluate_javascript(
            script,
            move |value: Result<JSValue, JavaScriptEvaluationError>| match value {
                Ok(JSValue::String(s)) => *result_clone.borrow_mut() = Some(Ok(s)),
                Ok(other) => {
                    *result_clone.borrow_mut() =
                        Some(Err(format!("expected string, got {other:?}")))
                }
                Err(e) => *result_clone.borrow_mut() = Some(Err(format!("{e:?}"))),
            },
        );

        let deadline = Instant::now() + timeout;
        loop {
            if let Some(result) = result.borrow().clone() {
                return result.map_err(BrowserError::Rendering);
            }
            if Instant::now() >= deadline {
                return Err(BrowserError::Timeout(
                    "waiting for evaluate_javascript".into(),
                ));
            }
            self.servo.spin_event_loop();
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}

struct PageHandle {
    loaded: Cell<bool>,
    frames: Cell<u32>,
}

impl PageHandle {
    fn new() -> Self {
        Self {
            loaded: Cell::new(false),
            frames: Cell::new(0),
        }
    }

    fn is_loaded(&self) -> bool {
        self.loaded.get()
    }

    fn mark_loaded(&self) {
        self.loaded.set(true);
    }

    fn frames(&self) -> u32 {
        self.frames.get()
    }

    fn increment_frames(&self) {
        self.frames.set(self.frames.get() + 1);
    }
}

struct PageDelegate {
    handle: Rc<PageHandle>,
}

impl PageDelegate {
    fn new() -> Self {
        Self {
            handle: Rc::new(PageHandle::new()),
        }
    }

    fn handle(&self) -> Rc<PageHandle> {
        self.handle.clone()
    }
}

impl WebViewDelegate for PageDelegate {
    fn notify_new_frame_ready(&self, webview: WebView) {
        log::debug!("notify_new_frame_ready called");
        webview.paint();
        self.handle.increment_frames();
    }

    fn notify_load_status_changed(&self, _webview: WebView, status: LoadStatus) {
        log::debug!("notify_load_status_changed: {status:?}");
        if matches!(status, LoadStatus::Complete) {
            self.handle.mark_loaded();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_config_default() {
        let config = BrowserConfig::default();
        assert_eq!(config.viewport_width, 1280);
        assert_eq!(config.viewport_height, 720);
    }
}
