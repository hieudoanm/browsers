mod font;

use browsers_css::Color;
use font::BitmapFont;
use image::{Rgba, RgbaImage};
use browsers_layout::Rect;
use browsers_paint::{DisplayCommand, DisplayList};

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub width: u32,
    pub height: u32,
    pub background: Color,
}

impl Default for RenderOptions {
    fn default() -> Self {
        RenderOptions {
            width: 800,
            height: 600,
            background: Color::WHITE,
        }
    }
}

pub struct SoftwareRenderer {
    buffer: RgbaImage,
    font: BitmapFont,
}

impl SoftwareRenderer {
    pub fn new(options: &RenderOptions) -> Self {
        SoftwareRenderer {
            buffer: RgbaImage::new(options.width, options.height),
            font: BitmapFont::new(),
        }
    }

    pub fn render(&mut self, list: &DisplayList, options: &RenderOptions) {
        // Fill background
        let bg = color_to_rgba(options.background);
        for pixel in self.buffer.pixels_mut() {
            *pixel = bg;
        }

        // Execute display commands
        for cmd in &list.commands {
            match cmd {
                DisplayCommand::FillRect { rect, color } => {
                    self.fill_rect(rect, *color);
                }
                DisplayCommand::StrokeRect { rect, color, width } => {
                    self.stroke_rect(rect, *color, *width);
                }
                DisplayCommand::DrawText {
                    rect,
                    text,
                    color,
                    font_size,
                } => {
                    self.draw_text(rect, text, *color, *font_size);
                }
            }
        }
    }

    fn fill_rect(&mut self, rect: &Rect, color: Color) {
        let rgba = color_to_rgba(color);
        let x0 = rect.x.round() as i32;
        let y0 = rect.y.round() as i32;
        let x1 = rect.right().round() as i32;
        let y1 = rect.bottom().round() as i32;

        for y in y0..y1 {
            for x in x0..x1 {
                if x >= 0
                    && x < self.buffer.width() as i32
                    && y >= 0
                    && y < self.buffer.height() as i32
                {
                    self.buffer.put_pixel(x as u32, y as u32, rgba);
                }
            }
        }
    }

    fn stroke_rect(&mut self, rect: &Rect, color: Color, width: f32) {
        let rgba = color_to_rgba(color);
        let w = width.round() as i32;
        let x0 = rect.x.round() as i32;
        let y0 = rect.y.round() as i32;
        let x1 = rect.right().round() as i32;
        let y1 = rect.bottom().round() as i32;

        for y in y0..y1 {
            for x in x0..x1 {
                let is_border =
                    (y - y0 < w) || (y1 - y - 1 < w) || (x - x0 < w) || (x1 - x - 1 < w);
                if is_border
                    && x >= 0
                    && x < self.buffer.width() as i32
                    && y >= 0
                    && y < self.buffer.height() as i32
                {
                    self.buffer.put_pixel(x as u32, y as u32, rgba);
                }
            }
        }
    }

    fn draw_text(&mut self, rect: &Rect, text: &str, color: Color, _font_size: f32) {
        let rgba = color_to_rgba(color);
        let start_x = rect.x.round() as i32;
        let start_y = rect.y.round() as i32;
        let glyph_w = self.font.glyph_width() as i32;
        let glyph_h = self.font.glyph_height() as i32;
        let max_width = rect.width.round() as i32;
        let max_chars_per_line = if glyph_w > 0 { (max_width / glyph_w).max(1) } else { 80 };

        let all_chars: Vec<char> = text.chars().collect();
        let mut pos = 0;
        let mut line_y = 0;

        while pos < all_chars.len() {
            let remaining_count = all_chars.len() - pos;
            if remaining_count <= max_chars_per_line as usize {
                for (i, &ch) in all_chars[pos..].iter().enumerate() {
                    let x = start_x + i as i32 * glyph_w;
                    let y = start_y + line_y;
                    self.font.draw_glyph(&mut self.buffer, ch, x, y, rgba);
                }
                break;
            }
            let end = (pos + max_chars_per_line as usize).min(all_chars.len());
            let mut break_at = end;
            let chunk = &all_chars[pos..end];
            if let Some(last_space) = chunk.iter().rposition(|&c| c == ' ') {
                break_at = pos + last_space + 1;
            }
            for (i, &ch) in all_chars[pos..break_at].iter().enumerate() {
                let x = start_x + i as i32 * glyph_w;
                let y = start_y + line_y;
                self.font.draw_glyph(&mut self.buffer, ch, x, y, rgba);
            }
            pos = break_at;
            line_y += glyph_h;
        }
    }

    pub fn into_buffer(self) -> RgbaImage {
        self.buffer
    }

    pub fn save_png(&self, path: &str) -> Result<(), image::ImageError> {
        self.buffer.save(path)
    }

    pub fn buffer(&self) -> &RgbaImage {
        &self.buffer
    }
}

fn color_to_rgba(color: Color) -> Rgba<u8> {
    Rgba([color.r, color.g, color.b, color.a])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_render_options() {
        let opts = RenderOptions::default();
        assert_eq!(opts.width, 800);
        assert_eq!(opts.height, 600);
    }

    #[test]
    fn create_renderer() {
        let opts = RenderOptions::default();
        let renderer = SoftwareRenderer::new(&opts);
        assert_eq!(renderer.buffer.width(), 800);
        assert_eq!(renderer.buffer.height(), 600);
    }

    #[test]
    fn render_empty_list() {
        let opts = RenderOptions::default();
        let mut renderer = SoftwareRenderer::new(&opts);
        let list = DisplayList::new();
        renderer.render(&list, &opts);
        // Background should be filled
        let pixel = renderer.buffer.get_pixel(0, 0);
        assert_eq!(pixel[0], 255);
        assert_eq!(pixel[1], 255);
        assert_eq!(pixel[2], 255);
    }

    #[test]
    fn render_fill_rect() {
        let opts = RenderOptions::default();
        let mut renderer = SoftwareRenderer::new(&opts);
        let mut list = DisplayList::new();
        list.push(DisplayCommand::FillRect {
            rect: Rect::new(10.0, 10.0, 50.0, 50.0),
            color: Color::new(255, 0, 0, 255),
        });
        renderer.render(&list, &opts);
        let pixel = renderer.buffer.get_pixel(20, 20);
        assert_eq!(pixel[0], 255);
        assert_eq!(pixel[1], 0);
        assert_eq!(pixel[2], 0);
    }

    #[test]
    fn render_out_of_bounds_rect() {
        let opts = RenderOptions::default();
        let mut renderer = SoftwareRenderer::new(&opts);
        let mut list = DisplayList::new();
        list.push(DisplayCommand::FillRect {
            rect: Rect::new(-10.0, -10.0, 5.0, 5.0),
            color: Color::new(255, 0, 0, 255),
        });
        renderer.render(&list, &opts);
        // Should not panic
    }

    #[test]
    fn color_conversion() {
        let c = Color::new(128, 64, 32, 200);
        let rgba = color_to_rgba(c);
        assert_eq!(rgba[0], 128);
        assert_eq!(rgba[1], 64);
        assert_eq!(rgba[2], 32);
        assert_eq!(rgba[3], 200);
    }
}
