use browsers_dom::{Dom, NodeId};
use browsers_style::{Display, Length, StyleContext};

#[derive(Debug, Clone)]
pub struct LayoutBox {
    pub node_id: NodeId,
    pub box_type: BoxType,
    pub children: Vec<LayoutBox>,
    pub rect: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxType {
    Block,
    Inline,
    Text,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect { x, y, width, height }
    }
    pub fn right(&self) -> f32 { self.x + self.width }
    pub fn bottom(&self) -> f32 { self.y + self.height }
}

#[derive(Debug, Default)]
pub struct LayoutContext {
    pub viewport_width: f32,
    pub viewport_height: f32,
}

impl LayoutContext {
    pub fn new(width: f32, height: f32) -> Self {
        LayoutContext { viewport_width: width, viewport_height: height }
    }
}

fn resolve_length(l: &Length, font_size: f32, ctx: &LayoutContext) -> f32 {
    l.to_px(font_size, ctx.viewport_width, ctx.viewport_height)
}

pub fn build_layout_tree(dom: &Dom, styles: &StyleContext) -> Vec<LayoutBox> {
    let mut boxes = Vec::new();
    let body = find_body(dom);
    if let Some(body) = body {
        if let Some(body_box) = build_box(dom, styles, body, BoxType::Block) {
            boxes.push(body_box);
        }
    }
    boxes
}

fn find_body(dom: &Dom) -> Option<NodeId> {
    let root = dom.root();
    let html = dom.children(root).iter()
        .find(|&&c| dom.element_name(c) == Some("html"))
        .copied()?;
    find_element_recursive(dom, html, "body")
}

fn find_element_recursive(dom: &Dom, start: NodeId, name: &str) -> Option<NodeId> {
    for &child in dom.children(start) {
        if dom.element_name(child) == Some(name) {
            return Some(child);
        }
        if let Some(found) = find_element_recursive(dom, child, name) {
            return Some(found);
        }
    }
    None
}

#[allow(dead_code)]
fn heading_font_size(tag: &str) -> f32 {
    match tag {
        "h1" => 32.0,
        "h2" => 24.0,
        "h3" => 18.72,
        "h4" => 16.0,
        "h5" => 13.28,
        "h6" => 10.72,
        _ => 16.0,
    }
}

fn build_box(
    dom: &Dom,
    styles: &StyleContext,
    node_id: NodeId,
    _containing: BoxType,
) -> Option<LayoutBox> {
    let style = styles.get(node_id);
    if style.display == Display::None || style.visibility != browsers_style::Visibility::Visible {
        return None;
    }
    let is_element = dom.is_element(node_id);
    let is_text = dom.is_text(node_id);
    if !is_element && !is_text {
        return None;
    }

    let box_type = if is_text {
        BoxType::Text
    } else {
        match style.display {
            Display::Block => BoxType::Block,
            Display::Inline | Display::InlineBlock => BoxType::Inline,
            _ => BoxType::Block,
        }
    };

    let mut children = Vec::new();
    for &child_id in dom.children(node_id) {
        if let Some(child_box) = build_box(dom, styles, child_id, box_type) {
            children.push(child_box);
        }
    }

    Some(LayoutBox { node_id, box_type, children, rect: Rect::default() })
}

pub fn layout(boxes: &mut [LayoutBox], ctx: &LayoutContext, dom: &Dom, styles: &StyleContext) {
    let mut y = 0.0f32;
    for box_ in boxes.iter_mut() {
        layout_block(box_, ctx, dom, styles, 0.0, ctx.viewport_width, &mut y);
    }
}

fn layout_block(
    box_: &mut LayoutBox,
    ctx: &LayoutContext,
    dom: &Dom,
    styles: &StyleContext,
    x: f32,
    available_width: f32,
    y: &mut f32,
) {
    let style = styles.get(box_.node_id);
    let parent_font_size = 16.0;
    let font_size = resolve_length(&style.font_size, parent_font_size, ctx);

    let margin_top = resolve_length(&style.margin_top, font_size, ctx);
    let margin_bottom = resolve_length(&style.margin_bottom, font_size, ctx);
    let padding_top = resolve_length(&style.padding_top, font_size, ctx);
    let padding_bottom = resolve_length(&style.padding_bottom, font_size, ctx);
    let border_top = resolve_length(&style.border_top_width, font_size, ctx);
    let border_bottom = resolve_length(&style.border_bottom_width, font_size, ctx);
    let border_left = resolve_length(&style.border_left_width, font_size, ctx);
    let border_right = resolve_length(&style.border_right_width, font_size, ctx);
    let padding_left = resolve_length(&style.padding_left, font_size, ctx);
    let padding_right = resolve_length(&style.padding_right, font_size, ctx);

    *y += margin_top;

    let inner_available = available_width
        - border_left - border_right - padding_left - padding_right;
    let content_width = match &style.width {
        Length::Auto => inner_available.max(0.0),
        w => resolve_length(w, font_size, ctx).min(inner_available).max(0.0),
    };

    let mut margin_left = resolve_length(&style.margin_left, font_size, ctx);
    let mut margin_right = resolve_length(&style.margin_right, font_size, ctx);
    let is_auto_left = matches!(style.margin_left, Length::Auto);
    let is_auto_right = matches!(style.margin_right, Length::Auto);
    if is_auto_left && is_auto_right {
        let used = content_width + border_left + border_right + padding_left + padding_right;
        let extra = (available_width - used).max(0.0);
        margin_left = extra / 2.0;
        margin_right = extra / 2.0;
    } else if is_auto_left {
        let used = margin_right + content_width + border_left + border_right + padding_left + padding_right;
        margin_left = (available_width - used).max(0.0);
    } else if is_auto_right {
        let used = margin_left + content_width + border_left + border_right + padding_left + padding_right;
        margin_right = (available_width - used).max(0.0);
    }
    let _ = margin_right;

    let content_x = x + margin_left + border_left + padding_left;

    box_.rect.x = x + margin_left;
    box_.rect.y = *y;
    box_.rect.width = content_width + padding_left + padding_right + border_left + border_right;

    let mut content_y = *y + border_top + padding_top;
    let start_y = content_y;

    for child in box_.children.iter_mut() {
        match child.box_type {
            BoxType::Block => {
                layout_block(child, ctx, dom, styles, content_x, content_width, &mut content_y);
            }
            BoxType::Inline | BoxType::Text => {
                layout_inline(child, ctx, dom, styles, content_x, content_width, &mut content_y);
            }
        }
    }

    let content_height = match &style.height {
        Length::Auto => (content_y - start_y).max(0.0),
        h => resolve_length(h, font_size, ctx),
    };

    box_.rect.height = content_height + padding_top + padding_bottom + border_top + border_bottom;
    *y += box_.rect.height + margin_bottom;
}

fn layout_inline(
    box_: &mut LayoutBox,
    ctx: &LayoutContext,
    dom: &Dom,
    styles: &StyleContext,
    x: f32,
    available_width: f32,
    y: &mut f32,
) {
    let style = styles.get(box_.node_id);
    let parent_font_size = 16.0;
    let font_size = resolve_length(&style.font_size, parent_font_size, ctx);

    if dom.is_text(box_.node_id) {
        let text = dom.text_content(box_.node_id).unwrap_or("");
        let text_height = font_size * 1.2;
        let char_width = font_size * 0.6;
        let chars_per_line = if char_width > 0.0 { (available_width / char_width).floor() as usize } else { 80 };
        let chars_per_line = chars_per_line.max(1);

        let mut total_height = 0.0f32;
        let all_chars: Vec<char> = text.chars().collect();
        let mut pos = 0;

        while pos < all_chars.len() {
            let remaining_count = all_chars.len() - pos;
            if remaining_count <= chars_per_line {
                pos += remaining_count;
                if pos < all_chars.len() || total_height > 0.0 {
                    // Only add line height for lines after the first
                    if total_height > 0.0 {
                        *y += text_height;
                    }
                    total_height += text_height;
                } else {
                    total_height = text_height;
                }
                break;
            }
            let end = (pos + chars_per_line).min(all_chars.len());
            let mut break_at = end;
            let chunk = &all_chars[pos..end];
            if let Some(last_space) = chunk.iter().rposition(|&c| c == ' ') {
                break_at = pos + last_space + 1;
            }
            pos = break_at;
            if total_height > 0.0 {
                *y += text_height;
            }
            total_height += text_height;
        }

        if total_height == 0.0 {
            total_height = text_height;
        }

        let total_text_width = all_chars.len() as f32 * char_width;
        box_.rect.x = x;
        box_.rect.y = *y;
        box_.rect.width = total_text_width.min(available_width);
        box_.rect.height = total_height;
        *y += total_height;
    } else {
        let mut inline_x = 0.0f32;
        let mut max_height = 0.0f32;
        let mut max_width = 0.0f32;

        for child in &mut box_.children {
            match child.box_type {
                BoxType::Inline | BoxType::Text => {
                    layout_inline(child, ctx, dom, styles, x + inline_x, available_width - inline_x, y);
                    inline_x += child.rect.width;
                    max_height = max_height.max(child.rect.height);
                    max_width = max_width.max(inline_x);
                }
                _ => {}
            }
        }

        let width = if inline_x > 0.0 { inline_x } else {
            resolve_length(&style.width, font_size, ctx)
        };
        let height = if max_height > 0.0 { max_height } else { font_size * 1.2 };

        box_.rect.x = x;
        box_.rect.y = *y;
        box_.rect.width = width.min(available_width).max(0.0);
        box_.rect.height = height;
    }
}

#[allow(dead_code)]
fn estimate_text_width(text: &str, font_size: f32) -> f32 {
    let char_width = font_size * 0.6;
    text.chars().count() as f32 * char_width
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_basics() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(r.right(), 110.0);
        assert_eq!(r.bottom(), 70.0);
    }

    #[test]
    fn layout_context() {
        let ctx = LayoutContext::new(1024.0, 768.0);
        assert_eq!(ctx.viewport_width, 1024.0);
        assert_eq!(ctx.viewport_height, 768.0);
    }

    #[test]
    fn build_empty_layout_tree() {
        let dom = Dom::new();
        let styles = StyleContext::new();
        let boxes = build_layout_tree(&dom, &styles);
        assert!(boxes.is_empty());
    }

    #[test]
    fn build_layout_tree_with_div() {
        let mut dom = Dom::new();
        let root = dom.root();
        let html = dom.create_element("html", "", root);
        let body = dom.create_element("body", "", html);
        dom.create_element("div", "", body);

        let styles = StyleContext::new();
        let boxes = build_layout_tree(&dom, &styles);
        assert_eq!(boxes.len(), 1);
    }

    #[test]
    fn test_estimate_text_width() {
        let w = estimate_text_width("hello", 16.0);
        assert!(w > 0.0);
    }

    #[test]
    fn layout_single_block() {
        let mut dom = Dom::new();
        let root = dom.root();
        let html = dom.create_element("html", "", root);
        let body = dom.create_element("body", "", html);
        dom.create_element("div", "", body);

        let styles = StyleContext::new();
        let ctx = LayoutContext::new(800.0, 600.0);
        let mut boxes = build_layout_tree(&dom, &styles);
        layout(&mut boxes, &ctx, &dom, &styles);

        assert!(!boxes.is_empty());
        assert!(boxes[0].rect.width > 0.0);
    }
}
