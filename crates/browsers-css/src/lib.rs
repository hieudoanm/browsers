use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Specificity {
    pub a: u32,
    pub b: u32,
    pub c: u32,
}

impl Specificity {
    pub fn new(a: u32, b: u32, c: u32) -> Self {
        Specificity { a, b, c }
    }
}

impl Ord for Specificity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.a
            .cmp(&other.a)
            .then(self.b.cmp(&other.b))
            .then(self.c.cmp(&other.c))
    }
}

impl PartialOrd for Specificity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stylesheet {
    pub rules: Vec<CssRule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CssRule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    pub parts: Vec<SelectorPart>,
}

impl Selector {
    pub fn specificity(&self) -> Specificity {
        let mut a = 0u32;
        let mut b = 0u32;
        let mut c = 0u32;

        for part in &self.parts {
            match &part.kind {
                SelectorKind::Id(_) => a += 1,
                SelectorKind::Class(_) | SelectorKind::Universal => b += 1,
                SelectorKind::Type(_) => c += 1,
            }
        }

        Specificity::new(a, b, c)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectorPart {
    pub kind: SelectorKind,
    pub combinator: Option<Combinator>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectorKind {
    Universal,
    Type(String),
    Class(String),
    Id(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    Descendant,
    Child,
    AdjacentSibling,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: Property,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Property {
    Color(Color),
    BackgroundColor(Color),
    Width(Length),
    Height(Length),
    Margin(Box<Length>),
    MarginTop(Length),
    MarginRight(Length),
    MarginBottom(Length),
    MarginLeft(Length),
    Padding(Box<Length>),
    PaddingTop(Length),
    PaddingRight(Length),
    PaddingBottom(Length),
    PaddingLeft(Length),
    Border(Border),
    FontSize(Length),
    FontFamily(String),
    FontWeight(FontWeight),
    Display(Display),
    TextAlign(TextAlign),
    TextDecoration(TextDecoration),
    Visibility(Visibility),
    Opacity(f32),
    Unknown(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> Option<Color> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                Some(Color { r, g, b, a: 255 })
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color { r, g, b, a: 255 })
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Color { r, g, b, a })
            }
            _ => None,
        }
    }

    pub fn from_name(name: &str) -> Option<Color> {
        match name.to_ascii_lowercase().as_str() {
            "black" => Some(Color::BLACK),
            "white" => Some(Color::WHITE),
            "red" => Some(Color::new(255, 0, 0, 255)),
            "green" => Some(Color::new(0, 128, 0, 255)),
            "blue" => Some(Color::new(0, 0, 255, 255)),
            "transparent" => Some(Color::TRANSPARENT),
            "gray" | "grey" => Some(Color::new(128, 128, 128, 255)),
            "lightgray" | "lightgrey" => Some(Color::new(211, 211, 211, 255)),
            "darkgray" | "darkgrey" => Some(Color::new(169, 169, 169, 255)),
            "silver" => Some(Color::new(192, 192, 192, 255)),
            "yellow" => Some(Color::new(255, 255, 0, 255)),
            "cyan" => Some(Color::new(0, 255, 255, 255)),
            "magenta" | "fuchsia" => Some(Color::new(255, 0, 255, 255)),
            "orange" => Some(Color::new(255, 165, 0, 255)),
            "purple" => Some(Color::new(128, 0, 128, 255)),
            "pink" => Some(Color::new(255, 192, 203, 255)),
            "brown" => Some(Color::new(165, 42, 42, 255)),
            "navy" => Some(Color::new(0, 0, 128, 255)),
            "teal" => Some(Color::new(0, 128, 128, 255)),
            "olive" => Some(Color::new(128, 128, 0, 255)),
            "maroon" => Some(Color::new(128, 0, 0, 255)),
            "lime" => Some(Color::new(0, 255, 0, 255)),
            "aqua" => Some(Color::new(0, 255, 255, 255)),
            _ => None,
        }
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a: 255 }
    }

    pub fn from_rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Color {
            r,
            g,
            b,
            a: (a * 255.0) as u8,
        }
    }

    pub fn from_rgb_string(s: &str) -> Option<Color> {
        let s = s.trim();
        if let Some(inner) = s.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() == 3 {
                let r = parts[0].parse::<u8>().ok()?;
                let g = parts[1].parse::<u8>().ok()?;
                let b = parts[2].parse::<u8>().ok()?;
                return Some(Color::from_rgb(r, g, b));
            }
        }
        if let Some(inner) = s.strip_prefix("rgba(").and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() == 4 {
                let r = parts[0].parse::<u8>().ok()?;
                let g = parts[1].parse::<u8>().ok()?;
                let b = parts[2].parse::<u8>().ok()?;
                let a = parts[3].parse::<f32>().ok()?;
                return Some(Color::from_rgba(r, g, b, a));
            }
        }
        None
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.a == 255 {
            write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            write!(
                f,
                "#{:02x}{:02x}{:02x}{:02x}",
                self.r, self.g, self.b, self.a
            )
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    Px(f32),
    Em(f32),
    Rem(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
    Auto,
}

impl Length {
    pub fn px(val: f32) -> Self {
        Length::Px(val)
    }

    pub fn value(&self) -> f32 {
        match self {
            Length::Px(v) => *v,
            _ => 0.0,
        }
    }

    pub fn to_px(&self, font_size: f32, viewport_width: f32, viewport_height: f32) -> f32 {
        match self {
            Length::Px(v) => *v,
            Length::Em(v) => *v * font_size,
            Length::Rem(v) => *v * 16.0,
            Length::Percent(v) => *v,
            Length::Vw(v) => *v * viewport_width / 100.0,
            Length::Vh(v) => *v * viewport_height / 100.0,
            Length::Auto => 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Border {
    pub width: Length,
    pub style: BorderStyle,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    None,
    Solid,
    Dashed,
    Dotted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Normal,
    Bold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Display {
    Block,
    Inline,
    InlineBlock,
    None,
    Flex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDecoration {
    None,
    Underline,
    Overline,
    LineThrough,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Hidden,
    Collapse,
}

pub fn parse_css(input: &str) -> Stylesheet {
    let mut rules = Vec::new();
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Stylesheet { rules };
    }

    let chars: Vec<char> = trimmed.chars().collect();
    let len = chars.len();
    let mut pos = 0;

    while pos < len {
        while pos < len && chars[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= len {
            break;
        }

        let selector_start = pos;
        while pos < len && chars[pos] != '{' {
            pos += 1;
        }
        if pos >= len {
            break;
        }
        let selector_str: String = chars[selector_start..pos].iter().collect();
        let selector_str = selector_str.trim();

        pos += 1;

        let body_start = pos;
        let mut depth = 1i32;
        while pos < len && depth > 0 {
            match chars[pos] {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
            if depth > 0 {
                pos += 1;
            }
        }
        let body: String = chars[body_start..pos].iter().collect();
        let body = body.trim();

        pos += 1;

        if let Some(rule) = parse_rule(selector_str, body) {
            rules.push(rule);
        }
    }

    Stylesheet { rules }
}

fn parse_rule(selector_str: &str, body: &str) -> Option<CssRule> {
    let selectors = parse_selectors(selector_str);
    if selectors.is_empty() {
        return None;
    }

    let declarations = parse_declarations(body);
    Some(CssRule {
        selectors,
        declarations,
    })
}

fn parse_selectors(input: &str) -> Vec<Selector> {
    input
        .split(',')
        .filter_map(|s| {
            let s = s.trim();
            if s.is_empty() {
                return None;
            }
            let parts = parse_selector_parts(s);
            if parts.is_empty() {
                None
            } else {
                Some(Selector { parts })
            }
        })
        .collect()
}

fn parse_selector_parts(input: &str) -> Vec<SelectorPart> {
    let mut parts = Vec::new();
    for token in input.split_whitespace() {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }

        let kind = if token == "*" {
            SelectorKind::Universal
        } else if let Some(rest) = token.strip_prefix('.') {
            SelectorKind::Class(rest.to_string())
        } else if let Some(rest) = token.strip_prefix('#') {
            SelectorKind::Id(rest.to_string())
        } else {
            SelectorKind::Type(token.to_string())
        };

        parts.push(SelectorPart {
            kind,
            combinator: None,
        });
    }
    parts
}

pub fn parse_declarations(body: &str) -> Vec<Declaration> {
    let mut declarations = Vec::new();
    for part in body.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((name, value)) = part.split_once(':') {
            let name = name.trim().to_string();
            let value = value.trim().to_string();

            match name.as_str() {
                "margin" => {
                    let props = parse_margin_shorthand(&value);
                    for prop in props {
                        declarations.push(Declaration {
                            name: name.clone(),
                            value: prop,
                        });
                    }
                }
                "padding" => {
                    let props = parse_padding_shorthand(&value);
                    for prop in props {
                        declarations.push(Declaration {
                            name: name.clone(),
                            value: prop,
                        });
                    }
                }
                _ => {
                    if let Some(prop) = parse_property(&name, &value) {
                        declarations.push(Declaration { name, value: prop });
                    }
                }
            }
        }
    }
    declarations
}

fn parse_color(value: &str) -> Option<Color> {
    Color::from_hex(value)
        .or_else(|| Color::from_name(value))
        .or_else(|| Color::from_rgb_string(value))
}

fn parse_margin_shorthand(value: &str) -> Vec<Property> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    let lengths: Vec<Length> = parts.iter().filter_map(|s| parse_length(s)).collect();

    match lengths.len() {
        1 => vec![
            Property::MarginTop(lengths[0]),
            Property::MarginRight(lengths[0]),
            Property::MarginBottom(lengths[0]),
            Property::MarginLeft(lengths[0]),
        ],
        2 => vec![
            Property::MarginTop(lengths[0]),
            Property::MarginRight(lengths[1]),
            Property::MarginBottom(lengths[0]),
            Property::MarginLeft(lengths[1]),
        ],
        3 => vec![
            Property::MarginTop(lengths[0]),
            Property::MarginRight(lengths[1]),
            Property::MarginBottom(lengths[2]),
            Property::MarginLeft(lengths[1]),
        ],
        4 => vec![
            Property::MarginTop(lengths[0]),
            Property::MarginRight(lengths[1]),
            Property::MarginBottom(lengths[2]),
            Property::MarginLeft(lengths[3]),
        ],
        _ => vec![],
    }
}

fn parse_padding_shorthand(value: &str) -> Vec<Property> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    let lengths: Vec<Length> = parts.iter().filter_map(|s| parse_length(s)).collect();

    match lengths.len() {
        1 => vec![
            Property::PaddingTop(lengths[0]),
            Property::PaddingRight(lengths[0]),
            Property::PaddingBottom(lengths[0]),
            Property::PaddingLeft(lengths[0]),
        ],
        2 => vec![
            Property::PaddingTop(lengths[0]),
            Property::PaddingRight(lengths[1]),
            Property::PaddingBottom(lengths[0]),
            Property::PaddingLeft(lengths[1]),
        ],
        3 => vec![
            Property::PaddingTop(lengths[0]),
            Property::PaddingRight(lengths[1]),
            Property::PaddingBottom(lengths[2]),
            Property::PaddingLeft(lengths[1]),
        ],
        4 => vec![
            Property::PaddingTop(lengths[0]),
            Property::PaddingRight(lengths[1]),
            Property::PaddingBottom(lengths[2]),
            Property::PaddingLeft(lengths[3]),
        ],
        _ => vec![],
    }
}

fn parse_property(name: &str, value: &str) -> Option<Property> {
    match name {
        "color" => parse_color(value).map(Property::Color),
        "background-color" | "background" => parse_color(value).map(Property::BackgroundColor),
        "width" => parse_length(value).map(Property::Width),
        "height" => parse_length(value).map(Property::Height),
        "margin" => {
            let props = parse_margin_shorthand(value);
            if props.len() == 1 {
                props.into_iter().next()
            } else {
                None
            }
        }
        "margin-top" => parse_length(value).map(Property::MarginTop),
        "margin-right" => parse_length(value).map(Property::MarginRight),
        "margin-bottom" => parse_length(value).map(Property::MarginBottom),
        "margin-left" => parse_length(value).map(Property::MarginLeft),
        "padding" => {
            let props = parse_padding_shorthand(value);
            if props.len() == 1 {
                props.into_iter().next()
            } else {
                None
            }
        }
        "padding-top" => parse_length(value).map(Property::PaddingTop),
        "padding-right" => parse_length(value).map(Property::PaddingRight),
        "padding-bottom" => parse_length(value).map(Property::PaddingBottom),
        "padding-left" => parse_length(value).map(Property::PaddingLeft),
        "border" => parse_border(value).map(Property::Border),
        "font-size" => parse_length(value).map(Property::FontSize),
        "font-family" => Some(Property::FontFamily(value.trim_matches('"').to_string())),
        "font-weight" => match value {
            "bold" => Some(Property::FontWeight(FontWeight::Bold)),
            _ => Some(Property::FontWeight(FontWeight::Normal)),
        },
        "display" => match value {
            "block" => Some(Property::Display(Display::Block)),
            "inline" => Some(Property::Display(Display::Inline)),
            "inline-block" => Some(Property::Display(Display::InlineBlock)),
            "none" => Some(Property::Display(Display::None)),
            "flex" => Some(Property::Display(Display::Flex)),
            _ => None,
        },
        "visibility" => match value {
            "visible" => Some(Property::Visibility(Visibility::Visible)),
            "hidden" => Some(Property::Visibility(Visibility::Hidden)),
            "collapse" => Some(Property::Visibility(Visibility::Collapse)),
            _ => None,
        },
        "text-align" => match value {
            "left" => Some(Property::TextAlign(TextAlign::Left)),
            "right" => Some(Property::TextAlign(TextAlign::Right)),
            "center" => Some(Property::TextAlign(TextAlign::Center)),
            "justify" => Some(Property::TextAlign(TextAlign::Justify)),
            _ => None,
        },
        "text-decoration" => match value {
            "none" => Some(Property::TextDecoration(TextDecoration::None)),
            "underline" => Some(Property::TextDecoration(TextDecoration::Underline)),
            "overline" => Some(Property::TextDecoration(TextDecoration::Overline)),
            "line-through" => Some(Property::TextDecoration(TextDecoration::LineThrough)),
            _ => None,
        },
        "opacity" => value.parse::<f32>().ok().map(Property::Opacity),
        _ => Some(Property::Unknown(name.to_string())),
    }
}

fn parse_length(value: &str) -> Option<Length> {
    let value = value.trim();
    if value == "auto" {
        return Some(Length::Auto);
    }
    if let Some(v) = value.strip_suffix("px") {
        if let Ok(n) = v.trim().parse::<f32>() {
            return Some(Length::Px(n));
        }
    }
    if let Some(v) = value.strip_suffix("em") {
        if let Ok(n) = v.trim().parse::<f32>() {
            return Some(Length::Em(n));
        }
    }
    if let Some(v) = value.strip_suffix("rem") {
        if let Ok(n) = v.trim().parse::<f32>() {
            return Some(Length::Rem(n));
        }
    }
    if let Some(v) = value.strip_suffix('%') {
        if let Ok(n) = v.trim().parse::<f32>() {
            return Some(Length::Percent(n));
        }
    }
    if let Some(v) = value.strip_suffix("vw") {
        if let Ok(n) = v.trim().parse::<f32>() {
            return Some(Length::Vw(n));
        }
    }
    if let Some(v) = value.strip_suffix("vh") {
        if let Ok(n) = v.trim().parse::<f32>() {
            return Some(Length::Vh(n));
        }
    }
    None
}

fn parse_border(value: &str) -> Option<Border> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    let width = parts
        .first()
        .and_then(|s| parse_length(s))
        .unwrap_or(Length::Px(1.0));
    let style = parts
        .get(1)
        .map(|s| match *s {
            "solid" => BorderStyle::Solid,
            "dashed" => BorderStyle::Dashed,
            "dotted" => BorderStyle::Dotted,
            _ => BorderStyle::None,
        })
        .unwrap_or(BorderStyle::None);
    let color = parts
        .get(2)
        .and_then(|s| Color::from_hex(s).or_else(|| Color::from_name(s)))
        .unwrap_or(Color::BLACK);

    Some(Border {
        width,
        style,
        color,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_css() {
        let ss = parse_css("");
        assert!(ss.rules.is_empty());
    }

    #[test]
    fn parse_single_rule() {
        let ss = parse_css("body { background: white; }");
        assert_eq!(ss.rules.len(), 1);
    }

    #[test]
    fn parse_color_hex() {
        let c = Color::from_hex("#ff0000").unwrap();
        assert_eq!(c, Color::new(255, 0, 0, 255));
    }

    #[test]
    fn parse_color_hex_short() {
        let c = Color::from_hex("#f0f").unwrap();
        assert_eq!(c, Color::new(255, 0, 255, 255));
    }

    #[test]
    fn parse_color_name() {
        assert_eq!(Color::from_name("red"), Some(Color::new(255, 0, 0, 255)));
        assert_eq!(Color::from_name("blue"), Some(Color::new(0, 0, 255, 255)));
    }

    #[test]
    fn parse_length_px() {
        assert_eq!(parse_length("10px"), Some(Length::Px(10.0)));
        assert_eq!(parse_length("auto"), Some(Length::Auto));
    }

    #[test]
    fn parse_selector_type() {
        let selectors = parse_selectors("div");
        assert_eq!(selectors.len(), 1);
        assert_eq!(
            selectors[0].parts[0].kind,
            SelectorKind::Type("div".to_string())
        );
    }

    #[test]
    fn parse_selector_class() {
        let selectors = parse_selectors(".box");
        assert_eq!(selectors.len(), 1);
        assert_eq!(
            selectors[0].parts[0].kind,
            SelectorKind::Class("box".to_string())
        );
    }

    #[test]
    fn parse_selector_id() {
        let selectors = parse_selectors("#main");
        assert_eq!(selectors.len(), 1);
        assert_eq!(
            selectors[0].parts[0].kind,
            SelectorKind::Id("main".to_string())
        );
    }

    #[test]
    fn parse_multiple_selectors() {
        let selectors = parse_selectors("div, span");
        assert_eq!(selectors.len(), 2);
    }

    #[test]
    fn test_parse_declarations() {
        let decls = parse_declarations("color: red; font-size: 16px;");
        assert_eq!(decls.len(), 2);
    }

    #[test]
    fn test_rgb_color() {
        let c = Color::from_rgb_string("rgb(255, 128, 0)").unwrap();
        assert_eq!(c, Color::new(255, 128, 0, 255));
    }

    #[test]
    fn test_rgba_color() {
        let c = Color::from_rgb_string("rgba(255, 128, 0, 0.5)").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 127);
    }

    #[test]
    fn test_margin_shorthand() {
        let decls = parse_declarations("margin: 10px 20px;");
        assert_eq!(decls.len(), 4);
        assert_eq!(decls[0].value, Property::MarginTop(Length::Px(10.0)));
        assert_eq!(decls[1].value, Property::MarginRight(Length::Px(20.0)));
        assert_eq!(decls[2].value, Property::MarginBottom(Length::Px(10.0)));
        assert_eq!(decls[3].value, Property::MarginLeft(Length::Px(20.0)));
    }

    #[test]
    fn test_specificity() {
        let sel_type = Selector {
            parts: vec![SelectorPart {
                kind: SelectorKind::Type("div".to_string()),
                combinator: None,
            }],
        };
        let sel_class = Selector {
            parts: vec![SelectorPart {
                kind: SelectorKind::Class("box".to_string()),
                combinator: None,
            }],
        };
        let sel_id = Selector {
            parts: vec![SelectorPart {
                kind: SelectorKind::Id("main".to_string()),
                combinator: None,
            }],
        };

        assert!(sel_id.specificity() > sel_class.specificity());
        assert!(sel_class.specificity() > sel_type.specificity());
    }
}
