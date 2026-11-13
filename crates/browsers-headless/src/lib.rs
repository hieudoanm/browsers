use std::time::{Duration, Instant};

use browsers_core::{BrowserConfig, BrowserContext, BrowserError, Page};

#[derive(Debug, thiserror::Error)]
pub enum HeadlessError {
    #[error("browser error: {0}")]
    Browser(#[from] BrowserError),

    #[error("timeout after {0}ms")]
    Timeout(u64),

    #[error("navigation failed: {0}")]
    Navigation(String),
}

pub struct HeadlessConfig {
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub load_timeout_ms: u64,
    pub wait_after_load_ms: u64,
}

impl Default for HeadlessConfig {
    fn default() -> Self {
        Self {
            viewport_width: 1280,
            viewport_height: 720,
            load_timeout_ms: 30_000,
            wait_after_load_ms: 3_000,
        }
    }
}

pub struct HeadlessBrowser {
    ctx: BrowserContext,
    config: HeadlessConfig,
}

impl HeadlessBrowser {
    pub fn new(config: HeadlessConfig) -> Result<Self, HeadlessError> {
        let browser_config = BrowserConfig {
            viewport_width: config.viewport_width,
            viewport_height: config.viewport_height,
            ..Default::default()
        };

        let ctx = BrowserContext::new(browser_config)?;

        Ok(Self { ctx, config })
    }

    pub fn render_url(&self, url: &str) -> Result<Page, HeadlessError> {
        let page = self.ctx.new_page(url)?;

        page.wait_for_load(self.config.load_timeout_ms)?;

        // Spin the event loop for a while after load to allow rendering to complete
        if self.config.wait_after_load_ms > 0 {
            let deadline =
                std::time::Instant::now() + Duration::from_millis(self.config.wait_after_load_ms);
            while std::time::Instant::now() < deadline {
                self.ctx.spin_event_loop();
                std::thread::sleep(Duration::from_millis(16));
            }
        }

        Ok(page)
    }

    pub fn screenshot_url(&self, url: &str, output_path: &str) -> Result<(), HeadlessError> {
        let page = self.render_url(url)?;

        page.save_screenshot(output_path)?;

        if let Ok(img) = page.surface_readback() {
            log::debug!("surface readback: {}", Page::pixel_summary(&img));
        }
        Ok(())
    }

    pub fn dump_html(&self, url: &str) -> Result<String, HeadlessError> {
        let (page, _) = self.render_url_best_effort(url, self.config.load_timeout_ms)?;
        page.dump_html(Duration::from_secs(15))
            .map_err(HeadlessError::from)
    }

    pub fn scrape(&self, url: &str) -> Result<ScrapeResult, HeadlessError> {
        self.scrape_with_timeout(url, self.config.load_timeout_ms)
    }

    /// scrape_with_timeout is scrape with an explicit load timeout. Servo owns
    /// one event loop per process, so callers reuse a single browser and vary
    /// the deadline per request instead of building a new browser.
    pub fn scrape_with_timeout(
        &self,
        url: &str,
        load_timeout_ms: u64,
    ) -> Result<ScrapeResult, HeadlessError> {
        let started = Instant::now();
        let peak_before = peak_rss_kb();
        let (page, timed_out) = self.render_url_best_effort(url, load_timeout_ms)?;
        let timeout = Duration::from_secs(15);

        let html = page.dump_html(timeout).map_err(HeadlessError::from)?;
        let final_url = page
            .evaluate_string("location.href", timeout)
            .unwrap_or_else(|_| url.to_string());
        let title = page
            .evaluate_string("document.title", timeout)
            .unwrap_or_default();

        let memory_kb = peak_rss_kb().saturating_sub(peak_before);
        Ok(ScrapeResult {
            html,
            url: final_url,
            title,
            timed_out,
            duration_ms: started.elapsed().as_millis(),
            memory_kb,
        })
    }

    pub fn screenshot_bytes(&self, url: &str) -> Result<ScreenshotResult, HeadlessError> {
        self.screenshot_bytes_with_timeout(url, self.config.load_timeout_ms)
    }

    /// screenshot_bytes_with_timeout is screenshot_bytes with an explicit load
    /// timeout, for callers that reuse one browser across requests.
    pub fn screenshot_bytes_with_timeout(
        &self,
        url: &str,
        load_timeout_ms: u64,
    ) -> Result<ScreenshotResult, HeadlessError> {
        use std::io::Cursor;

        let started = Instant::now();
        let peak_before = peak_rss_kb();
        let (page, timed_out) = self.render_url_best_effort(url, load_timeout_ms)?;
        let timeout = Duration::from_secs(15);

        let img = page.screenshot()?;
        let mut png: Vec<u8> = Vec::new();
        img.write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
            .map_err(|e| HeadlessError::Browser(BrowserError::Rendering(e.to_string())))?;

        let final_url = page
            .evaluate_string("location.href", timeout)
            .unwrap_or_else(|_| url.to_string());
        let title = page
            .evaluate_string("document.title", timeout)
            .unwrap_or_default();

        let memory_kb = peak_rss_kb().saturating_sub(peak_before);
        Ok(ScreenshotResult {
            png,
            url: final_url,
            title,
            timed_out,
            duration_ms: started.elapsed().as_millis(),
            memory_kb,
        })
    }

    fn render_url_best_effort(
        &self,
        url: &str,
        load_timeout_ms: u64,
    ) -> Result<(Page, bool), HeadlessError> {
        let page = self.ctx.new_page(url)?;

        let timed_out = match page.wait_for_load(load_timeout_ms) {
            Ok(()) => false,
            Err(BrowserError::Timeout(_)) => {
                log::warn!("load did not complete before timeout; dumping partial DOM");
                true
            }
            Err(other) => return Err(other.into()),
        };

        // Let the script thread settle before evaluating JavaScript.
        self.ctx.spin_event_loop();
        Ok((page, timed_out))
    }

    pub fn spin_event_loop(&self) {
        self.ctx.spin_event_loop();
    }
}

fn peak_rss_kb() -> u64 {
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    if unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) } != 0 {
        return 0;
    }
    #[cfg(target_os = "macos")]
    {
        usage.ru_maxrss as u64 / 1024
    }
    #[cfg(target_os = "linux")]
    {
        usage.ru_maxrss as u64
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = usage;
        0
    }
}

pub struct ScrapeResult {
    pub html: String,
    pub url: String,
    pub title: String,
    pub timed_out: bool,
    pub duration_ms: u128,
    pub memory_kb: u64,
}

pub struct ScreenshotResult {
    pub png: Vec<u8>,
    pub url: String,
    pub title: String,
    pub timed_out: bool,
    pub duration_ms: u128,
    pub memory_kb: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headless_config_default() {
        let config = HeadlessConfig::default();
        assert_eq!(config.viewport_width, 1280);
        assert_eq!(config.viewport_height, 720);
        assert_eq!(config.load_timeout_ms, 30_000);
    }
}
