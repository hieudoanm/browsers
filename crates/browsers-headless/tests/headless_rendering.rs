use browsers_headless::{HeadlessBrowser, HeadlessConfig};

/// Servo's `Opts`/`Preferences` are process-global and can only be initialized once,
/// so this entire pipeline is exercised in a single test.
#[test]
fn headless_rendering_pipeline() {
    let fixture_dir =
        std::env::temp_dir().join(format!("browsers-headless-test-{}", std::process::id()));
    std::fs::create_dir_all(&fixture_dir).unwrap();

    let red = fixture_dir.join("red.html");
    std::fs::write(
        &red,
        "<!doctype html><html><head><style>body { \
         margin: 0; background: red; \
         }</style></head><body></body></html>",
    )
    .unwrap();

    let config = HeadlessConfig {
        viewport_width: 320,
        viewport_height: 180,
        load_timeout_ms: 60_000,
        wait_after_load_ms: 300,
    };
    let browser = HeadlessBrowser::new(config).expect("browser init");

    // Viewport is applied to both the WebView and the rendering context.
    let page = browser
        .render_url(&format!("file://{}", red.display()))
        .expect("render red");
    assert_eq!(page.surface_size(), (320, 180));
    assert_eq!(page.webview_size(), (320.0, 180.0));

    // The drawing surface contains the actual document content.
    let img = page.surface_readback().expect("surface readback");
    assert_eq!((img.width(), img.height()), (320, 180));
    let pixel = img.get_pixel(10, 10);
    assert_eq!(
        (pixel[0], pixel[1], pixel[2], pixel[3]),
        (255, 0, 0, 255),
        "surface should render the red page background, not an empty page"
    );

    // The take_screenshot pipeline writes the same content to disk.
    let out = fixture_dir.join("red.png");
    page.save_screenshot(out.to_str().unwrap())
        .expect("save screenshot");

    let img = image::open(&out).unwrap().to_rgba8();
    let pixel = img.get_pixel(10, 10);
    assert_eq!(
        (pixel[0], pixel[1], pixel[2], pixel[3]),
        (255, 0, 0, 255),
        "saved PNG should render the red page background"
    );
}
