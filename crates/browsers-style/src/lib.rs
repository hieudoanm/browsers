pub use browsers_css::{
    Color, Display, FontWeight, Length, Property, SelectorKind, TextAlign, TextDecoration,
    Visibility,
};
use browsers_dom::{Dom, NodeId};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ComputedStyle {
    pub color: Color,
    pub background_color: Color,
    pub width: Length,
    pub height: Length,
    pub margin_top: Length,
    pub margin_right: Length,
    pub margin_bottom: Length,
    pub margin_left: Length,
    pub padding_top: Length,
    pub padding_right: Length,
    pub padding_bottom: Length,
    pub padding_left: Length,
    pub border_top_width: Length,
    pub border_right_width: Length,
    pub border_bottom_width: Length,
    pub border_left_width: Length,
    pub border_top_color: Color,
    pub border_right_color: Color,
    pub border_bottom_color: Color,
    pub border_left_color: Color,
    pub font_size: Length,
    pub font_family: String,
    pub font_weight: FontWeight,
    pub display: Display,
    pub text_align: TextAlign,
    pub text_decoration: TextDecoration,
    pub visibility: Visibility,
    pub opacity: f32,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        ComputedStyle {
            color: Color::BLACK,
            background_color: Color::TRANSPARENT,
            width: Length::Auto,
            height: Length::Auto,
            margin_top: Length::Px(0.0),
            margin_right: Length::Px(0.0),
            margin_bottom: Length::Px(0.0),
            margin_left: Length::Px(0.0),
            padding_top: Length::Px(0.0),
            padding_right: Length::Px(0.0),
            padding_bottom: Length::Px(0.0),
            padding_left: Length::Px(0.0),
            border_top_width: Length::Px(0.0),
            border_right_width: Length::Px(0.0),
            border_bottom_width: Length::Px(0.0),
            border_left_width: Length::Px(0.0),
            border_top_color: Color::BLACK,
            border_right_color: Color::BLACK,
            border_bottom_color: Color::BLACK,
            border_left_color: Color::BLACK,
            font_size: Length::Px(16.0),
            font_family: "serif".to_string(),
            font_weight: FontWeight::Normal,
            display: Display::Block,
            text_align: TextAlign::Left,
            text_decoration: TextDecoration::None,
            visibility: Visibility::Visible,
            opacity: 1.0,
        }
    }
}

impl ComputedStyle {
    pub fn inherited(&self) -> Self {
        ComputedStyle {
            color: self.color,
            font_size: self.font_size,
            font_family: self.font_family.clone(),
            font_weight: self.font_weight,
            visibility: self.visibility,
            opacity: self.opacity,
            ..ComputedStyle::default()
        }
    }
}

#[derive(Debug, Default)]
pub struct StyleContext {
    styles: HashMap<NodeId, ComputedStyle>,
    default: ComputedStyle,
}

impl StyleContext {
    pub fn new() -> Self {
        StyleContext {
            styles: HashMap::new(),
            default: ComputedStyle::default(),
        }
    }

    pub fn compute(&mut self, dom: &Dom, stylesheet: &browsers_css::Stylesheet) {
        self.compute_node(dom, dom.root(), stylesheet, &ComputedStyle::default());
    }

    fn compute_node(
        &mut self,
        dom: &Dom,
        node_id: NodeId,
        stylesheet: &browsers_css::Stylesheet,
        parent_style: &ComputedStyle,
    ) {
        let mut style = parent_style.inherited();

        // Collect all matching rules with their specificity
        let mut matching_rules: Vec<(browsers_css::Specificity, &browsers_css::CssRule)> = Vec::new();
        for rule in &stylesheet.rules {
            if self.matches_selector(dom, &rule.selectors, node_id) {
                let specificity = rule
                    .selectors
                    .iter()
                    .map(|s| s.specificity())
                    .max()
                    .unwrap_or(browsers_css::Specificity::new(0, 0, 0));
                matching_rules.push((specificity, rule));
            }
        }

        // Sort by specificity (lower first, so higher specificity overwrites)
        matching_rules.sort_by(|a, b| a.0.cmp(&b.0));

        // Apply stylesheet rules in specificity order
        for (_, rule) in matching_rules {
            for decl in &rule.declarations {
                self.apply_declaration(&mut style, &decl.value);
            }
        }

        // Apply inline styles from style attribute (highest priority)
        if let Some(style_attr) = dom.attribute(node_id, "style") {
            for decl in browsers_css::parse_declarations(style_attr) {
                self.apply_declaration(&mut style, &decl.value);
            }
        }

        self.styles.insert(node_id, style.clone());

        for &child in dom.children(node_id) {
            self.compute_node(dom, child, stylesheet, &style);
        }
    }

    fn matches_selector(&self, dom: &Dom, selectors: &[browsers_css::Selector], node_id: NodeId) -> bool {
        selectors
            .iter()
            .any(|sel| self.matches_single_selector(dom, sel, node_id))
    }

    fn matches_single_selector(
        &self,
        dom: &Dom,
        selector: &browsers_css::Selector,
        node_id: NodeId,
    ) -> bool {
        if selector.parts.is_empty() {
            return false;
        }
        self.matches_last_part(dom, selector, selector.parts.len() - 1, node_id)
    }

    fn matches_last_part(
        &self,
        dom: &Dom,
        selector: &browsers_css::Selector,
        part_idx: usize,
        node_id: NodeId,
    ) -> bool {
        let part = &selector.parts[part_idx];
        if !self.matches_part(dom, &part.kind, node_id) {
            return false;
        }

        if part_idx == 0 {
            return true;
        }

        let parent = match dom.parent(node_id) {
            Some(p) => p,
            None => return false,
        };

        let combinator = selector.parts[part_idx - 1].combinator;
        match combinator {
            Some(browsers_css::Combinator::Child) => {
                self.matches_last_part(dom, selector, part_idx - 1, parent)
            }
            Some(browsers_css::Combinator::Descendant) | None => {
                let mut current = parent;
                loop {
                    if self.matches_last_part(dom, selector, part_idx - 1, current) {
                        return true;
                    }
                    match dom.parent(current) {
                        Some(p) => current = p,
                        None => return false,
                    }
                }
            }
            _ => false,
        }
    }

    fn matches_part(&self, dom: &Dom, kind: &SelectorKind, node_id: NodeId) -> bool {
        match kind {
            SelectorKind::Universal => dom.is_element(node_id),
            SelectorKind::Type(name) => dom.element_name(node_id) == Some(name.as_str()),
            SelectorKind::Class(name) => {
                if let Some(class_attr) = dom.attribute(node_id, "class") {
                    class_attr.split_whitespace().any(|c| c == name.as_str())
                } else {
                    false
                }
            }
            SelectorKind::Id(name) => dom.attribute(node_id, "id") == Some(name.as_str()),
        }
    }

    fn apply_declaration(&self, style: &mut ComputedStyle, prop: &Property) {
        match prop {
            Property::Color(c) => style.color = *c,
            Property::BackgroundColor(c) => style.background_color = *c,
            Property::Width(l) => style.width = *l,
            Property::Height(l) => style.height = *l,
            Property::Margin(l) => {
                style.margin_top = *l.clone();
                style.margin_right = *l.clone();
                style.margin_bottom = *l.clone();
                style.margin_left = *l.clone();
            }
            Property::MarginTop(l) => style.margin_top = *l,
            Property::MarginRight(l) => style.margin_right = *l,
            Property::MarginBottom(l) => style.margin_bottom = *l,
            Property::MarginLeft(l) => style.margin_left = *l,
            Property::Padding(l) => {
                style.padding_top = *l.clone();
                style.padding_right = *l.clone();
                style.padding_bottom = *l.clone();
                style.padding_left = *l.clone();
            }
            Property::PaddingTop(l) => style.padding_top = *l,
            Property::PaddingRight(l) => style.padding_right = *l,
            Property::PaddingBottom(l) => style.padding_bottom = *l,
            Property::PaddingLeft(l) => style.padding_left = *l,
            Property::Border(b) => {
                style.border_top_width = b.width;
                style.border_right_width = b.width;
                style.border_bottom_width = b.width;
                style.border_left_width = b.width;
                style.border_top_color = b.color;
                style.border_right_color = b.color;
                style.border_bottom_color = b.color;
                style.border_left_color = b.color;
            }
            Property::FontSize(l) => style.font_size = *l,
            Property::FontFamily(f) => style.font_family = f.clone(),
            Property::FontWeight(w) => style.font_weight = *w,
            Property::Display(d) => style.display = *d,
            Property::TextAlign(a) => style.text_align = *a,
            Property::TextDecoration(d) => style.text_decoration = *d,
            Property::Visibility(v) => style.visibility = *v,
            Property::Opacity(o) => style.opacity = *o,
            Property::Unknown(_) => {}
        }
    }

    pub fn get(&self, node_id: NodeId) -> &ComputedStyle {
        self.styles.get(&node_id).unwrap_or(&self.default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use browsers_css::{Declaration, Selector, SelectorKind, SelectorPart, Stylesheet};

    fn make_stylesheet(declarations: Vec<Property>) -> browsers_css::Stylesheet {
        Stylesheet {
            rules: vec![browsers_css::CssRule {
                selectors: vec![Selector {
                    parts: vec![SelectorPart {
                        kind: SelectorKind::Universal,
                        combinator: None,
                    }],
                }],
                declarations: declarations
                    .into_iter()
                    .enumerate()
                    .map(|(i, p)| Declaration {
                        name: format!("prop{i}"),
                        value: p,
                    })
                    .collect(),
            }],
        }
    }

    #[test]
    fn default_style() {
        let style = ComputedStyle::default();
        assert_eq!(style.color, Color::BLACK);
        assert_eq!(style.display, Display::Block);
        assert_eq!(style.opacity, 1.0);
    }

    #[test]
    fn compute_text_color() {
        let mut dom = Dom::new();
        let root = dom.root();
        let div = dom.create_element("div", "", root);

        let ss = make_stylesheet(vec![Property::Color(Color::new(255, 0, 0, 255))]);
        let mut ctx = StyleContext::new();
        ctx.compute(&dom, &ss);
        assert_eq!(ctx.get(div).color, Color::new(255, 0, 0, 255));
    }

    #[test]
    fn matches_class_selector() {
        let mut dom = Dom::new();
        let root = dom.root();
        let div = dom.create_element("div", "", root);
        dom.set_attribute(div, "class", "box");

        let ss = Stylesheet {
            rules: vec![browsers_css::CssRule {
                selectors: vec![Selector {
                    parts: vec![SelectorPart {
                        kind: SelectorKind::Class("box".to_string()),
                        combinator: None,
                    }],
                }],
                declarations: vec![Declaration {
                    name: "color".to_string(),
                    value: Property::Color(Color::new(0, 255, 0, 255)),
                }],
            }],
        };

        let mut ctx = StyleContext::new();
        ctx.compute(&dom, &ss);
        assert_eq!(ctx.get(div).color, Color::new(0, 255, 0, 255));
    }

    #[test]
    fn matches_id_selector() {
        let mut dom = Dom::new();
        let root = dom.root();
        let div = dom.create_element("div", "", root);
        dom.set_attribute(div, "id", "main");

        let ss = Stylesheet {
            rules: vec![browsers_css::CssRule {
                selectors: vec![Selector {
                    parts: vec![SelectorPart {
                        kind: SelectorKind::Id("main".to_string()),
                        combinator: None,
                    }],
                }],
                declarations: vec![Declaration {
                    name: "font-size".to_string(),
                    value: Property::FontSize(Length::Px(20.0)),
                }],
            }],
        };

        let mut ctx = StyleContext::new();
        ctx.compute(&dom, &ss);
        assert_eq!(ctx.get(div).font_size, Length::Px(20.0));
    }

    #[test]
    fn cascading_styles() {
        let mut dom = Dom::new();
        let root = dom.root();
        let div = dom.create_element("div", "", root);
        dom.set_attribute(div, "class", "box");

        let ss = Stylesheet {
            rules: vec![
                browsers_css::CssRule {
                    selectors: vec![Selector {
                        parts: vec![SelectorPart {
                            kind: SelectorKind::Universal,
                            combinator: None,
                        }],
                    }],
                    declarations: vec![Declaration {
                        name: "color".to_string(),
                        value: Property::Color(Color::BLACK),
                    }],
                },
                browsers_css::CssRule {
                    selectors: vec![Selector {
                        parts: vec![SelectorPart {
                            kind: SelectorKind::Class("box".to_string()),
                            combinator: None,
                        }],
                    }],
                    declarations: vec![Declaration {
                        name: "color".to_string(),
                        value: Property::Color(Color::WHITE),
                    }],
                },
            ],
        };

        let mut ctx = StyleContext::new();
        ctx.compute(&dom, &ss);
        assert_eq!(ctx.get(div).color, Color::WHITE);
    }

    #[test]
    fn inherits_color_from_parent() {
        let mut dom = Dom::new();
        let root = dom.root();
        let parent = dom.create_element("div", "", root);
        let child = dom.create_element("span", "", parent);

        let ss = Stylesheet {
            rules: vec![browsers_css::CssRule {
                selectors: vec![Selector {
                    parts: vec![SelectorPart {
                        kind: SelectorKind::Type("div".to_string()),
                        combinator: None,
                    }],
                }],
                declarations: vec![Declaration {
                    name: "color".to_string(),
                    value: Property::Color(Color::new(255, 0, 0, 255)),
                }],
            }],
        };

        let mut ctx = StyleContext::new();
        ctx.compute(&dom, &ss);
        assert_eq!(ctx.get(parent).color, Color::new(255, 0, 0, 255));
        assert_eq!(ctx.get(child).color, Color::new(255, 0, 0, 255));
    }

    #[test]
    fn inherits_font_size() {
        let mut dom = Dom::new();
        let root = dom.root();
        let parent = dom.create_element("div", "", root);
        let child = dom.create_element("span", "", parent);

        let ss = Stylesheet {
            rules: vec![browsers_css::CssRule {
                selectors: vec![Selector {
                    parts: vec![SelectorPart {
                        kind: SelectorKind::Type("div".to_string()),
                        combinator: None,
                    }],
                }],
                declarations: vec![Declaration {
                    name: "font-size".to_string(),
                    value: Property::FontSize(Length::Px(24.0)),
                }],
            }],
        };

        let mut ctx = StyleContext::new();
        ctx.compute(&dom, &ss);
        assert_eq!(ctx.get(parent).font_size, Length::Px(24.0));
        assert_eq!(ctx.get(child).font_size, Length::Px(24.0));
    }

    #[test]
    fn child_overrides_inherited() {
        let mut dom = Dom::new();
        let root = dom.root();
        let parent = dom.create_element("div", "", root);
        let child = dom.create_element("span", "", parent);
        dom.set_attribute(child, "class", "highlight");

        let ss = Stylesheet {
            rules: vec![
                browsers_css::CssRule {
                    selectors: vec![Selector {
                        parts: vec![SelectorPart {
                            kind: SelectorKind::Type("div".to_string()),
                            combinator: None,
                        }],
                    }],
                    declarations: vec![Declaration {
                        name: "color".to_string(),
                        value: Property::Color(Color::new(255, 0, 0, 255)),
                    }],
                },
                browsers_css::CssRule {
                    selectors: vec![Selector {
                        parts: vec![SelectorPart {
                            kind: SelectorKind::Class("highlight".to_string()),
                            combinator: None,
                        }],
                    }],
                    declarations: vec![Declaration {
                        name: "color".to_string(),
                        value: Property::Color(Color::new(0, 255, 0, 255)),
                    }],
                },
            ],
        };

        let mut ctx = StyleContext::new();
        ctx.compute(&dom, &ss);
        assert_eq!(ctx.get(parent).color, Color::new(255, 0, 0, 255));
        assert_eq!(ctx.get(child).color, Color::new(0, 255, 0, 255));
    }

    #[test]
    fn background_not_inherited() {
        let mut dom = Dom::new();
        let root = dom.root();
        let parent = dom.create_element("div", "", root);
        let child = dom.create_element("span", "", parent);

        let ss = Stylesheet {
            rules: vec![browsers_css::CssRule {
                selectors: vec![Selector {
                    parts: vec![SelectorPart {
                        kind: SelectorKind::Type("div".to_string()),
                        combinator: None,
                    }],
                }],
                declarations: vec![Declaration {
                    name: "background-color".to_string(),
                    value: Property::BackgroundColor(Color::new(0, 0, 255, 255)),
                }],
            }],
        };

        let mut ctx = StyleContext::new();
        ctx.compute(&dom, &ss);
        assert_eq!(ctx.get(parent).background_color, Color::new(0, 0, 255, 255));
        assert_eq!(ctx.get(child).background_color, Color::TRANSPARENT);
    }

    #[test]
    fn deep_inheritance() {
        let mut dom = Dom::new();
        let root = dom.root();
        let grandparent = dom.create_element("div", "", root);
        let parent = dom.create_element("div", "", grandparent);
        let child = dom.create_element("span", "", parent);

        let ss = Stylesheet {
            rules: vec![browsers_css::CssRule {
                selectors: vec![Selector {
                    parts: vec![SelectorPart {
                        kind: SelectorKind::Type("div".to_string()),
                        combinator: None,
                    }],
                }],
                declarations: vec![
                    Declaration {
                        name: "color".to_string(),
                        value: Property::Color(Color::new(100, 100, 100, 255)),
                    },
                    Declaration {
                        name: "font-size".to_string(),
                        value: Property::FontSize(Length::Px(20.0)),
                    },
                ],
            }],
        };

        let mut ctx = StyleContext::new();
        ctx.compute(&dom, &ss);
        assert_eq!(ctx.get(grandparent).color, Color::new(100, 100, 100, 255));
        assert_eq!(ctx.get(parent).color, Color::new(100, 100, 100, 255));
        assert_eq!(ctx.get(child).color, Color::new(100, 100, 100, 255));
        assert_eq!(ctx.get(grandparent).font_size, Length::Px(20.0));
        assert_eq!(ctx.get(parent).font_size, Length::Px(20.0));
        assert_eq!(ctx.get(child).font_size, Length::Px(20.0));
    }
}
