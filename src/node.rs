use std::str;

use crate::{
    error::Error,
    page::{Page, INTERNAL_NODE_HEADER_SIZE, INTERNAL_NODE_NUM_CHILDREN_OFFSET, IS_ROOT_OFFSET, LEAF_NODE_HEADER_SIZE, LEAF_NODE_NUM_PAIRS_OFFSET, NODE_TYPE_OFFSET, PARENT_POINTER_OFFSET, PTR_SIZE},
    utils::byte_to_bool
};

pub const KEY_SIZE: usize = 10;
pub const VALUE_SIZE: usize = 10;

#[derive(Clone, Debug)]
pub struct Offset(pub usize);

impl TryFrom<[u8; PTR_SIZE]> for Offset {
    type Error = Error;

    fn try_from(arr: [u8; PTR_SIZE]) -> Result<Self, Self::Error> {
        Ok(Offset(usize::from_be_bytes(arr)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Key(pub String);

#[derive(Clone, Debug, Eq)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

impl KeyValuePair {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

impl Ord for KeyValuePair {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl PartialOrd for KeyValuePair {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for KeyValuePair {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
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

impl From<u8> for NodeType {
    fn from(value: u8) -> Self {
        match value {
            0x01 => NodeType::Internal(Vec::new(), Vec::new()),
            0x02 => NodeType::Leaf(Vec::new()),
            _ => NodeType::Unexpected,
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

    pub fn split(&mut self, branches: usize) -> Result<(Key, Node), Error> {
        match self.node_type {
            NodeType::Internal(ref mut children, ref mut keys) => {
                let mut sibling_keys = keys.split_off(branches - 1);
                let median_key = sibling_keys.remove(0);
                let sibling_children = children.split_off(branches);
                Ok((
                    median_key,
                    Node::new(NodeType::Internal(sibling_children, sibling_keys), false, self.parent_offset.clone())
                ))
            }
            NodeType::Leaf(ref mut pairs) => {
                let sibling_pairs = pairs.split_off(branches);
                let median_pair = pairs.get(branches - 1).ok_or(Error::UnexpectedError)?.clone();
                Ok((
                    Key(median_pair.key.clone()),
                    Node::new(NodeType::Leaf(sibling_pairs), false, self.parent_offset.clone())
                ))
            }
            NodeType::Unexpected => Err(Error::UnexpectedError),
        }
    }
}

impl TryFrom<Page> for Node {
    type Error = Error;

    fn try_from(value: Page) -> Result<Self, Self::Error> {
        let raw = value.get_data();
        let node_type = NodeType::from(raw[NODE_TYPE_OFFSET]);
        let is_root = byte_to_bool(raw[IS_ROOT_OFFSET]);
        let parent_offset: Option<Offset>;
        if is_root {
            parent_offset = None;
        } else {
            parent_offset = Some(Offset(value.get_value_from_offset(PARENT_POINTER_OFFSET)?));
        }

        match node_type {
            NodeType::Internal(mut children, mut keys) => {
                let num_children = value.get_value_from_offset(INTERNAL_NODE_NUM_CHILDREN_OFFSET)?;
                let mut offset = INTERNAL_NODE_HEADER_SIZE;

                for _i in 1..=num_children {
                    let child_offset = value.get_value_from_offset(offset)?;
                    children.push(Offset(child_offset));
                    offset += PTR_SIZE;
                }

                for _i in 1..num_children {
                    let key_raw = value.get_ptr_from_offset(offset, KEY_SIZE);
                    let Ok(key) = str::from_utf8(key_raw) else {
                        return Err(Error::UTF8Error);
                    };
                    offset += KEY_SIZE;
                    keys.push(Key(key.trim_matches(char::from(0)).to_string()));
                }
                Ok(Node::new(
                    NodeType::Internal(children, keys),
                    is_root,
                    parent_offset,
                ))
            }
            NodeType::Leaf(mut pairs) => {
                let mut offset = LEAF_NODE_NUM_PAIRS_OFFSET;
                let num_keys_val_pairs = value.get_value_from_offset(offset)?;
                offset = LEAF_NODE_HEADER_SIZE;

                for _i in 1..=num_keys_val_pairs {
                    let key_raw = value.get_ptr_from_offset(offset, KEY_SIZE);
                    let Ok(key) = str::from_utf8(key_raw) else {
                        return Err(Error::UTF8Error);
                    };
                    offset += KEY_SIZE;

                    let value_raw = value.get_ptr_from_offset(offset, VALUE_SIZE);
                    let Ok(value) = str::from_utf8(value_raw) else {
                        return Err(Error::UTF8Error);
                    };
                    offset += VALUE_SIZE;

                    pairs.push(
                        KeyValuePair::new(
                            key.trim_matches(char::from(0)).to_string(),
                            value.trim_matches(char::from(0)).to_string(),
                        ),
                    );
                }
                Ok(Node::new(NodeType::Leaf(pairs), is_root, parent_offset))
            }
            NodeType::Unexpected => Err(Error::UnexpectedError),
        }
    }
}
