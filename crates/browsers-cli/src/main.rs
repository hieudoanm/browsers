mod headed;

use browsers_cli::serve;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "browserverless")]
#[command(about = "Lightweight browser powered by Servo")]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Open a URL in headed mode (default)
    Open {
        /// URL to open (default: https://example.com)
        #[arg(default_value = "https://example.com")]
        url: String,
    },

    /// Run in headless mode
    Headless {
        /// URL to render
        url: String,

        /// Viewport width
        #[arg(long, default_value = "1280")]
        width: u32,

        /// Viewport height
        #[arg(long, default_value = "720")]
        height: u32,

        /// Load timeout in milliseconds
        #[arg(long, default_value = "30000")]
        timeout: u64,

        /// Wait after load in milliseconds
        #[arg(long, default_value = "3000")]
        wait: u64,
    },

    /// Take a screenshot of a URL
    Screenshot {
        /// URL to screenshot
        url: String,

        /// Output file path
        #[arg(short, long, default_value = "screenshot.png")]
        output: PathBuf,

        /// Viewport width
        #[arg(long, default_value = "1280")]
        width: u32,

        /// Viewport height
        #[arg(long, default_value = "720")]
        height: u32,

        /// Load timeout in milliseconds
        #[arg(long, default_value = "30000")]
        timeout: u64,
    },

    /// Scrape the full HTML of a URL
    Scrape {
        /// URL to dump
        url: String,

        /// Output file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Load timeout in milliseconds
        #[arg(long, default_value = "30000")]
        timeout: u64,
    },

    /// Load a URL headless and report its peak memory usage and duration
    Memory {
        /// URL to measure
        url: String,

        /// Viewport width
        #[arg(long, default_value = "1280")]
        width: u32,

        /// Viewport height
        #[arg(long, default_value = "720")]
        height: u32,

        /// Load timeout in milliseconds
        #[arg(long, default_value = "30000")]
        timeout: u64,

        /// Observe window after load in milliseconds
        #[arg(long, default_value = "5000")]
        observe: u64,
    },

    /// Model Context Protocol server exposing rendering as tools
    Mcp {
        #[command(subcommand)]
        command: McpCommand,
    },

    /// Start an HTTP server exposing the headless rendering web API
    Serve {
        /// Address to bind, e.g. 127.0.0.1:8080
        #[arg(long, default_value = "127.0.0.1:8080")]
        bind: String,

        /// Port to listen on; overrides the port in --bind
        #[arg(long)]
        port: Option<u16>,

        /// Viewport width
        #[arg(long, default_value = "1280")]
        width: u32,

        /// Viewport height
        #[arg(long, default_value = "720")]
        height: u32,

        /// Per-request load timeout in milliseconds
        #[arg(long, default_value = "30000")]
        timeout: u64,
    },
}

/// The `browserverless mcp` command tree.
#[derive(Subcommand)]
enum McpCommand {
    /// Start the browserverless MCP server on stdio
    Serve {
        /// Viewport width
        #[arg(long, default_value = "1280")]
        width: u32,

        /// Viewport height
        #[arg(long, default_value = "720")]
        height: u32,

        /// Load timeout in milliseconds
        #[arg(long, default_value = "30000")]
        timeout: u64,
    },
}

fn build_version() -> String {
    format!(
        "{} (build {})",
        env!("CARGO_PKG_VERSION"),
        env!("BROWSERS_BUILD_TIMESTAMP")
    )
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let matches = Cli::command().version(build_version()).get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|err| err.exit());

    match cli.command {
        Commands::Open { url: _ } => {
            headed::run_headed()?;
        }

        Commands::Headless {
            url,
            width,
            height,
            timeout,
            wait,
        } => {
            let config = browsers_headless::HeadlessConfig {
                viewport_width: width,
                viewport_height: height,
                load_timeout_ms: timeout,
                wait_after_load_ms: wait,
            };

            let browser = browsers_headless::HeadlessBrowser::new(config)?;
            let _page = browser.render_url(&url)?;

            println!("Page loaded successfully.");

            browser.spin_event_loop();
        }

        Commands::Screenshot {
            url,
            output,
            width,
            height,
            timeout,
        } => {
            let config = browsers_headless::HeadlessConfig {
                viewport_width: width,
                viewport_height: height,
                load_timeout_ms: timeout,
                wait_after_load_ms: 3000,
            };

            let browser = browsers_headless::HeadlessBrowser::new(config)?;
            browser.screenshot_url(&url, output.to_str().unwrap_or("screenshot.png"))?;

            println!("Screenshot saved to {}", output.display());

            browser.spin_event_loop();
        }

        Commands::Scrape {
            url,
            output,
            timeout,
        } => {
            let config = browsers_headless::HeadlessConfig {
                viewport_width: 1280,
                viewport_height: 720,
                load_timeout_ms: timeout,
                wait_after_load_ms: 1_000,
            };

            let browser = browsers_headless::HeadlessBrowser::new(config)?;
            let html = browser.dump_html(&url)?;

            match output {
                Some(path) => {
                    std::fs::write(&path, html)?;
                    println!("HTML dumped to {}", path.display());
                }
                None => println!("{html}"),
            }
        }

        Commands::Memory {
            url,
            width,
            height,
            timeout,
            observe,
        } => {
            let config = browsers_headless::HeadlessConfig {
                viewport_width: width,
                viewport_height: height,
                load_timeout_ms: timeout,
                wait_after_load_ms: 3_000,
            };

            let started = Instant::now();
            let browser = browsers_headless::HeadlessBrowser::new(config)?;
            browser.render_url(&url)?;

            // Keep the event loop spinning through the observe window so page
            // scripts and layout/paint continue allocating while we measure.
            let deadline = started + Duration::from_millis(observe);
            while Instant::now() < deadline {
                browser.spin_event_loop();
                std::thread::sleep(Duration::from_millis(16));
            }

            println!("peak RSS: {} MB", peak_rss_mb());
            println!("elapsed: {:.3} s", started.elapsed().as_secs_f64());
        }

        Commands::Mcp {
            command:
                McpCommand::Serve {
                    width,
                    height,
                    timeout,
                },
        } => {
            // Diagnostics go to stderr so stdout carries nothing but frames.
            browsers_cli::mcp::run(width, height, timeout, &mut std::io::stderr())?;
        }

        Commands::Serve {
            bind,
            port,
            width,
            height,
            timeout,
        } => {
            serve::run(serve::ServeConfig {
                bind,
                port,
                viewport_width: width,
                viewport_height: height,
                load_timeout_ms: timeout,
            })?;
        }
    }

    Ok(())
}

fn peak_rss_mb() -> u64 {
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    if unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) } != 0 {
        return 0;
    }
    #[cfg(target_os = "macos")]
    {
        usage.ru_maxrss as u64 / (1024 * 1024)
    }
    #[cfg(target_os = "linux")]
    {
        usage.ru_maxrss as u64 / 1024
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = usage;
        0
    }
}
