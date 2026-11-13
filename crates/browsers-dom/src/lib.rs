use std::collections::HashMap;
use std::fmt;

pub type NodeId = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Document(DocumentData),
    Element(ElementData),
    Text(TextData),
    Comment(CommentData),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentData {
    pub quirks_mode: QuirksMode,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ElementData {
    pub name: String,
    pub namespace: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextData {
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommentData {
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuirksMode {
    #[default]
    NoQuirks,
    Quirks,
    LimitedQuirks,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub node: Node,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
}

#[derive(Debug, Default)]
pub struct Dom {
    nodes: Vec<TreeNode>,
    root: NodeId,
}

impl Dom {
    pub fn new() -> Self {
        let doc = TreeNode {
            node: Node::Document(DocumentData {
                quirks_mode: QuirksMode::NoQuirks,
            }),
            parent: None,
            children: Vec::new(),
        };
        let mut dom = Dom {
            nodes: Vec::new(),
            root: 0,
        };
        dom.nodes.push(doc);
        dom
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn create_element(&mut self, name: &str, namespace: &str, parent: NodeId) -> NodeId {
        let id = self.nodes.len();
        let node = TreeNode {
            node: Node::Element(ElementData {
                name: name.to_ascii_lowercase(),
                namespace: namespace.to_string(),
                attributes: HashMap::new(),
            }),
            parent: Some(parent),
            children: Vec::new(),
        };
        self.nodes.push(node);
        self.nodes[parent].children.push(id);
        id
    }

    pub fn create_element_orphan(&mut self, name: &str, namespace: &str) -> NodeId {
        let id = self.nodes.len();
        let node = TreeNode {
            node: Node::Element(ElementData {
                name: name.to_ascii_lowercase(),
                namespace: namespace.to_string(),
                attributes: HashMap::new(),
            }),
            parent: None,
            children: Vec::new(),
        };
        self.nodes.push(node);
        id
    }

    pub fn create_text_orphan(&mut self, content: &str) -> NodeId {
        let id = self.nodes.len();
        let node = TreeNode {
            node: Node::Text(TextData {
                content: content.to_string(),
            }),
            parent: None,
            children: Vec::new(),
        };
        self.nodes.push(node);
        id
    }

    pub fn create_comment_orphan(&mut self, content: &str) -> NodeId {
        let id = self.nodes.len();
        let node = TreeNode {
            node: Node::Comment(CommentData {
                content: content.to_string(),
            }),
            parent: None,
            children: Vec::new(),
        };
        self.nodes.push(node);
        id
    }

    pub fn create_text(&mut self, content: &str, parent: NodeId) -> NodeId {
        let id = self.nodes.len();
        let node = TreeNode {
            node: Node::Text(TextData {
                content: content.to_string(),
            }),
            parent: Some(parent),
            children: Vec::new(),
        };
        self.nodes.push(node);
        self.nodes[parent].children.push(id);
        id
    }

    pub fn create_comment(&mut self, content: &str, parent: NodeId) -> NodeId {
        let id = self.nodes.len();
        let node = TreeNode {
            node: Node::Comment(CommentData {
                content: content.to_string(),
            }),
            parent: Some(parent),
            children: Vec::new(),
        };
        self.nodes.push(node);
        self.nodes[parent].children.push(id);
        id
    }

    pub fn set_attribute(&mut self, node_id: NodeId, name: &str, value: &str) {
        if let Node::Element(ref mut elem) = self.nodes[node_id].node {
            elem.attributes.insert(name.to_string(), value.to_string());
        }
    }

    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id).map(|n| &n.node)
    }

    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.nodes.get(id).and_then(|n| n.parent)
    }

    pub fn children(&self, id: NodeId) -> &[NodeId] {
        self.nodes.get(id).map_or(&[], |n| &n.children)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn element_name(&self, id: NodeId) -> Option<&str> {
        match &self.nodes[id].node {
            Node::Element(e) => Some(&e.name),
            _ => None,
        }
    }

    pub fn attribute(&self, id: NodeId, name: &str) -> Option<&str> {
        match &self.nodes[id].node {
            Node::Element(e) => e.attributes.get(name).map(|s| s.as_str()),
            _ => None,
        }
    }

    pub fn text_content(&self, id: NodeId) -> Option<&str> {
        match &self.nodes[id].node {
            Node::Text(t) => Some(&t.content),
            _ => None,
        }
    }

    pub fn is_element(&self, id: NodeId) -> bool {
        matches!(self.nodes[id].node, Node::Element(_))
    }

    pub fn is_text(&self, id: NodeId) -> bool {
        matches!(self.nodes[id].node, Node::Text(_))
    }

    pub fn append_child(&mut self, parent: NodeId, child: NodeId) {
        self.nodes[child].parent = Some(parent);
        self.nodes[parent].children.push(child);
    }
}

impl fmt::Display for Dom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_node(f, self.root, 0)
    }
}

impl Dom {
    fn write_node(&self, f: &mut fmt::Formatter<'_>, id: NodeId, indent: usize) -> fmt::Result {
        let node = &self.nodes[id];
        let pad = "  ".repeat(indent);
        match &node.node {
            Node::Document(_) => {
                writeln!(f, "{pad}Document")?;
            }
            Node::Element(e) => {
                write!(f, "{pad}<{}", e.name)?;
                for (k, v) in &e.attributes {
                    write!(f, " {k}=\"{v}\"")?;
                }
                writeln!(f, ">")?;
            }
            Node::Text(t) => {
                let trimmed = t.content.trim();
                if !trimmed.is_empty() {
                    writeln!(f, "{pad}\"{trimmed}\"")?;
                }
            }
            Node::Comment(c) => {
                writeln!(f, "{pad}<!-- {} -->", c.content)?;
            }
        }
        for &child in &node.children {
            self.write_node(f, child, indent + 1)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_dom_has_document_root() {
        let dom = Dom::new();
        assert!(matches!(dom.get(dom.root()), Some(Node::Document(_))));
    }

    #[test]
    fn create_element() {
        let mut dom = Dom::new();
        let root = dom.root();
        let div = dom.create_element("div", "http://www.w3.org/1999/xhtml", root);
        assert_eq!(dom.element_name(div), Some("div"));
        assert_eq!(dom.parent(div), Some(root));
        assert_eq!(dom.children(root), &[div]);
    }

    #[test]
    fn create_text() {
        let mut dom = Dom::new();
        let root = dom.root();
        let text = dom.create_text("hello", root);
        assert_eq!(dom.text_content(text), Some("hello"));
    }

    #[test]
    fn create_comment() {
        let mut dom = Dom::new();
        let root = dom.root();
        let comment = dom.create_comment("test", root);
        assert!(matches!(dom.get(comment), Some(Node::Comment(_))));
    }

    #[test]
    fn set_attribute() {
        let mut dom = Dom::new();
        let root = dom.root();
        let div = dom.create_element("div", "", root);
        dom.set_attribute(div, "class", "box");
        assert_eq!(dom.attribute(div, "class"), Some("box"));
    }

    #[test]
    fn nested_elements() {
        let mut dom = Dom::new();
        let root = dom.root();
        let body = dom.create_element("body", "", root);
        let div = dom.create_element("div", "", body);
        let text = dom.create_text("hello", div);
        assert_eq!(dom.children(div), &[text]);
        assert_eq!(dom.parent(text), Some(div));
    }

    #[test]
    fn node_count() {
        let mut dom = Dom::new();
        let root = dom.root();
        assert_eq!(dom.node_count(), 1);
        dom.create_element("html", "", root);
        assert_eq!(dom.node_count(), 2);
    }

    #[test]
    fn element_name_case_insensitive() {
        let mut dom = Dom::new();
        let root = dom.root();
        let div = dom.create_element("DIV", "", root);
        assert_eq!(dom.element_name(div), Some("div"));
    }
}
