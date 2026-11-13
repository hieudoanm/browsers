use std::borrow::Cow;
use std::cell::RefCell;

use browsers_dom::Dom;
use html5ever::tendril::StrTendril;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink};
use html5ever::{parse_document, Attribute, QualName};

pub fn parse_html(input: &str) -> Dom {
    let sink = DomSink::new();
    let parser = parse_document(sink, Default::default());
    let sink = parser
        .from_utf8()
        .read_from(&mut input.as_bytes())
        .expect("html5ever parse failed");
    sink.dom.into_inner()
}

struct DomSink {
    dom: RefCell<Dom>,
}

impl DomSink {
    fn new() -> Self {
        DomSink {
            dom: RefCell::new(Dom::new()),
        }
    }
}

/// Minimal ElemName wrapper for html5ever
struct DomElemName {
    ns: markup5ever::Namespace,
    local: markup5ever::LocalName,
}

impl std::fmt::Debug for DomElemName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{}", self.ns, self.local)
    }
}

impl html5ever::tree_builder::ElemName for DomElemName {
    fn ns(&self) -> &markup5ever::Namespace {
        &self.ns
    }
    fn local_name(&self) -> &markup5ever::LocalName {
        &self.local
    }
}

impl TreeSink for DomSink {
    type Handle = browsers_dom::NodeId;
    type Output = Self;
    type ElemName<'a> = DomElemName;

    fn finish(self) -> Self {
        self
    }

    fn parse_error(&self, _msg: Cow<'static, str>) {
        // Silently ignore parse errors — html5ever is very strict,
        // real browsers are lenient. We don't want to crash on malformed HTML.
    }

    fn get_document(&self) -> browsers_dom::NodeId {
        self.dom.borrow().root()
    }

    fn elem_name<'a>(&'a self, target: &'a browsers_dom::NodeId) -> DomElemName {
        let dom = self.dom.borrow();
        let name = dom
            .element_name(*target)
            .expect("elem_name called on non-element")
            .to_string();
        DomElemName {
            ns: markup5ever::ns!(html),
            local: name.into(),
        }
    }

    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<Attribute>,
        _flags: ElementFlags,
    ) -> browsers_dom::NodeId {
        let local_name = name.local.to_string();
        let ns = name.ns.as_ref();

        let mut dom = self.dom.borrow_mut();
        let elem = dom.create_element_orphan(&local_name, ns);

        for attr in attrs {
            let attr_name = attr.name.local.to_string();
            let attr_value = attr.value.to_string();
            dom.set_attribute(elem, &attr_name, &attr_value);
        }

        elem
    }

    fn create_comment(&self, text: StrTendril) -> browsers_dom::NodeId {
        let mut dom = self.dom.borrow_mut();
        dom.create_comment_orphan(&text)
    }

    fn create_pi(&self, _target: StrTendril, _data: StrTendril) -> browsers_dom::NodeId {
        // Processing instructions not supported — return a dummy node
        self.dom.borrow().root()
    }

    fn append(&self, parent: &browsers_dom::NodeId, child: NodeOrText<browsers_dom::NodeId>) {
        let mut dom = self.dom.borrow_mut();
        match child {
            NodeOrText::AppendNode(node) => {
                dom.append_child(*parent, node);
            }
            NodeOrText::AppendText(text) => {
                let text_str: String = text.into();
                if !text_str.trim().is_empty() {
                    let text_id = dom.create_text_orphan(&text_str);
                    dom.append_child(*parent, text_id);
                }
            }
        }
    }

    fn append_based_on_parent_node(
        &self,
        element: &browsers_dom::NodeId,
        _prev_element: &browsers_dom::NodeId,
        child: NodeOrText<browsers_dom::NodeId>,
    ) {
        self.append(element, child);
    }

    fn append_doctype_to_document(
        &self,
        _name: StrTendril,
        _public_id: StrTendril,
        _system_id: StrTendril,
    ) {
        // Doctype — ignored, we keep the existing document root
    }

    fn get_template_contents(&self, target: &browsers_dom::NodeId) -> browsers_dom::NodeId {
        *target
    }

    fn same_node(&self, x: &browsers_dom::NodeId, y: &browsers_dom::NodeId) -> bool {
        x == y
    }

    fn set_quirks_mode(&self, _mode: QuirksMode) {
        // Ignored for now
    }

    fn append_before_sibling(&self, _sibling: &browsers_dom::NodeId, _new_node: NodeOrText<browsers_dom::NodeId>) {
        // Not implemented — append to parent instead
    }

    fn add_attrs_if_missing(&self, target: &browsers_dom::NodeId, attrs: Vec<Attribute>) {
        let mut dom = self.dom.borrow_mut();
        for attr in attrs {
            let attr_name = attr.name.local.to_string();
            let attr_value = attr.value.to_string();
            if dom.attribute(*target, &attr_name).is_none() {
                dom.set_attribute(*target, &attr_name, &attr_value);
            }
        }
    }

    fn remove_from_parent(&self, _target: &browsers_dom::NodeId) {
        // Not implemented
    }

    fn reparent_children(&self, _node: &browsers_dom::NodeId, _new_parent: &browsers_dom::NodeId) {
        // Not implemented
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty() {
        let dom = parse_html("");
        assert!(dom.get(dom.root()).is_some());
    }

    #[test]
    fn parse_simple_element() {
        let dom = parse_html("<html><body><div></div></body></html>");
        let root = dom.root();
        let html = dom.children(root)[0];
        assert_eq!(dom.element_name(html), Some("html"));
    }

    #[test]
    fn parse_text_content() {
        let dom = parse_html("<html><body><p>hello</p></body></html>");
        let root = dom.root();
        let html = dom.children(root)[0];
        let body = dom
            .children(html)
            .iter()
            .find(|&&c| dom.element_name(c) == Some("body"))
            .copied()
            .unwrap();
        let p = dom
            .children(body)
            .iter()
            .find(|&&c| dom.element_name(c) == Some("p"))
            .copied()
            .unwrap();
        let text = dom.children(p)[0];
        assert_eq!(dom.text_content(text), Some("hello"));
    }

    #[test]
    fn parse_attributes() {
        let dom = parse_html(r#"<html><body><div class="box" id="main"></div></body></html>"#);
        let root = dom.root();
        let html = dom.children(root)[0];
        let body = dom
            .children(html)
            .iter()
            .find(|&&c| dom.element_name(c) == Some("body"))
            .copied()
            .unwrap();
        let div = dom
            .children(body)
            .iter()
            .find(|&&c| dom.element_name(c) == Some("div"))
            .copied()
            .unwrap();
        assert_eq!(dom.attribute(div, "class"), Some("box"));
        assert_eq!(dom.attribute(div, "id"), Some("main"));
    }

    #[test]
    fn parse_nested() {
        let dom = parse_html("<html><body><div><span>text</span></div></body></html>");
        let root = dom.root();
        let html = dom.children(root)[0];
        let body = dom
            .children(html)
            .iter()
            .find(|&&c| dom.element_name(c) == Some("body"))
            .copied()
            .unwrap();
        let div = dom
            .children(body)
            .iter()
            .find(|&&c| dom.element_name(c) == Some("div"))
            .copied()
            .unwrap();
        let span = dom
            .children(div)
            .iter()
            .find(|&&c| dom.element_name(c) == Some("span"))
            .copied()
            .unwrap();
        assert_eq!(dom.element_name(span), Some("span"));
    }

    #[test]
    fn parse_comment() {
        let dom = parse_html("<!-- hello -->");
        assert!(dom.node_count() > 1);
    }

    #[test]
    fn parse_void_element() {
        let dom = parse_html("<html><body><br><img src=\"test.png\"></body></html>");
        let root = dom.root();
        let html = dom.children(root)[0];
        let body = dom
            .children(html)
            .iter()
            .find(|&&c| dom.element_name(c) == Some("body"))
            .copied()
            .unwrap();
        let children = dom.children(body);
        assert!(children.len() >= 1);
    }

    #[test]
    fn parse_self_closing() {
        let dom = parse_html(r#"<html><body><input type="text" /></body></html>"#);
        assert!(dom.node_count() > 1);
    }

    #[test]
    fn parse_style_tag() {
        let dom = parse_html(
            r#"<html><head><style>body { color: red; }</style></head><body></body></html>"#,
        );
        let root = dom.root();
        let html = dom.children(root)[0];
        let head = dom
            .children(html)
            .iter()
            .find(|&&c| dom.element_name(c) == Some("head"))
            .copied()
            .unwrap();
        let style = dom
            .children(head)
            .iter()
            .find(|&&c| dom.element_name(c) == Some("style"))
            .copied()
            .unwrap();
        let text = dom.children(style)[0];
        assert_eq!(dom.text_content(text), Some("body { color: red; }"));
    }
}
