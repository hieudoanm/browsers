use browsers_css::Color;
use browsers_dom::Dom;
use browsers_layout::{LayoutBox, LayoutContext, Rect};
use browsers_style::StyleContext;

#[derive(Debug, Clone)]
pub enum DisplayCommand {
    FillRect {
        rect: Rect,
        color: Color,
    },
    StrokeRect {
        rect: Rect,
        color: Color,
        width: f32,
    },
    DrawText {
        rect: Rect,
        text: String,
        color: Color,
        font_size: f32,
    },
}

#[derive(Debug, Default)]
pub struct DisplayList {
    pub commands: Vec<DisplayCommand>,
}

impl DisplayList {
    pub fn new() -> Self {
        DisplayList {
            commands: Vec::new(),
        }
    }

    pub fn push(&mut self, cmd: DisplayCommand) {
        self.commands.push(cmd);
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }
}

pub fn build_display_list(boxes: &[LayoutBox], dom: &Dom, styles: &StyleContext, ctx: &LayoutContext) -> DisplayList {
    let mut list = DisplayList::new();
    for box_ in boxes {
        build_box_commands(box_, dom, styles, ctx, &mut list);
    }
    list
}

fn build_box_commands(box_: &LayoutBox, dom: &Dom, styles: &StyleContext, ctx: &LayoutContext, list: &mut DisplayList) {
    let style = styles.get(box_.node_id);

    // Draw background
    if style.background_color != Color::TRANSPARENT {
        let content_rect = content_rect(box_);
        list.push(DisplayCommand::FillRect {
            rect: content_rect,
            color: style.background_color,
        });
    }

    // Draw borders
    draw_borders(box_, style, list);

    // Draw text
    if dom.is_text(box_.node_id) {
        if let Some(text) = dom.text_content(box_.node_id) {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                let parent_font_size = 16.0;
                let font_size = style.font_size.to_px(
                    parent_font_size,
                    ctx.viewport_width,
                    ctx.viewport_height,
                );
                list.push(DisplayCommand::DrawText {
                    rect: box_.rect,
                    text: trimmed.to_string(),
                    color: style.color,
                    font_size,
                });
            }
        }
    }

    // Recurse into children
    for child in &box_.children {
        build_box_commands(child, dom, styles, ctx, list);
    }
}

fn content_rect(box_: &LayoutBox) -> Rect {
    let style_data = ComputedStyleRef {
        border_left: 0.0,
        border_top: 0.0,
        padding_left: 0.0,
        padding_top: 0.0,
    };

    Rect::new(
        box_.rect.x + style_data.border_left + style_data.padding_left,
        box_.rect.y + style_data.border_top + style_data.padding_top,
        box_.rect.width,
        box_.rect.height,
    )
}

struct ComputedStyleRef {
    border_left: f32,
    border_top: f32,
    padding_left: f32,
    padding_top: f32,
}

fn draw_borders(box_: &LayoutBox, style: &browsers_style::ComputedStyle, list: &mut DisplayList) {
    let top_width = style.border_top_width.value();
    let bottom_width = style.border_bottom_width.value();
    let left_width = style.border_left_width.value();
    let right_width = style.border_right_width.value();

    if top_width > 0.0 {
        list.push(DisplayCommand::FillRect {
            rect: Rect::new(box_.rect.x, box_.rect.y, box_.rect.width, top_width),
            color: style.border_top_color,
        });
    }
    if bottom_width > 0.0 {
        list.push(DisplayCommand::FillRect {
            rect: Rect::new(
                box_.rect.x,
                box_.rect.bottom() - bottom_width,
                box_.rect.width,
                bottom_width,
            ),
            color: style.border_bottom_color,
        });
    }
    if left_width > 0.0 {
        list.push(DisplayCommand::FillRect {
            rect: Rect::new(
                box_.rect.x,
                box_.rect.y + top_width,
                left_width,
                box_.rect.height - top_width - bottom_width,
            ),
            color: style.border_left_color,
        });
    }
    if right_width > 0.0 {
        list.push(DisplayCommand::FillRect {
            rect: Rect::new(
                box_.rect.right() - right_width,
                box_.rect.y + top_width,
                right_width,
                box_.rect.height - top_width - bottom_width,
            ),
            color: style.border_right_color,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use browsers_layout::BoxType;

    #[test]
    fn empty_display_list() {
        let list = DisplayList::new();
        assert!(list.is_empty());
    }

    #[test]
    fn display_list_push() {
        let mut list = DisplayList::new();
        list.push(DisplayCommand::FillRect {
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            color: Color::WHITE,
        });
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn build_empty_display_list() {
        let dom = Dom::new();
        let styles = StyleContext::new();
        let ctx = browsers_layout::LayoutContext::new(800.0, 600.0);
        let list = build_display_list(&[], &dom, &styles, &ctx);
        assert!(list.is_empty());
    }

    #[test]
    fn build_display_list_with_element() {
        let mut dom = Dom::new();
        let root = dom.root();
        let div = dom.create_element("div", "", root);

        let styles = StyleContext::new();
        let box_ = LayoutBox {
            node_id: div,
            box_type: BoxType::Block,
            children: Vec::new(),
            rect: Rect::new(0.0, 0.0, 100.0, 50.0),
        };

        let ctx = browsers_layout::LayoutContext::new(800.0, 600.0);
        let list = build_display_list(&[box_], &dom, &styles, &ctx);
        assert!(list.is_empty());
    }

    #[test]
    fn content_rect_calculation() {
        let box_ = LayoutBox {
            node_id: 0,
            box_type: BoxType::Block,
            children: Vec::new(),
            rect: Rect::new(10.0, 20.0, 100.0, 50.0),
        };
        let r = content_rect(&box_);
        assert_eq!(r.x, 10.0);
        assert_eq!(r.y, 20.0);
    }
}
