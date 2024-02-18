pub const KEY_SIZE: usize = 10;
pub const VALUE_SIZE: usize = 10;

#[derive(Clone, Debug)]
pub struct Offset(pub usize);

#[derive(Clone, Debug)]
pub struct Key(pub String);

#[derive(Clone, Debug)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

impl KeyValuePair {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

#[derive(Clone, Debug)]
pub enum NodeType {
    Internal(Vec<Offset>, Vec<Key>),
    Leaf(Vec<KeyValuePair>),
    Unexpected,
}

impl From<&NodeType> for u8 {
    fn from(value: &NodeType) -> Self {
        match value {
            NodeType::Internal(_, _) => 0x01,
            NodeType::Leaf(_) => 0x02,
            NodeType::Unexpected => 0x03,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub node_type: NodeType,
    pub is_root: bool,
    pub parent_offset: Option<Offset>,
}

impl Node {
    pub fn new(node_type: NodeType, is_root: bool, parent_offset: Option<Offset>) -> Self {
        Self {
            node_type,
            is_root,
            parent_offset,
        }
    }
}
