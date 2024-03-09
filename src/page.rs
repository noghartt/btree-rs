use std::mem::size_of;

use crate::{
    error::Error,
    node::{Key, Node, NodeType, Offset, KEY_SIZE, VALUE_SIZE},
    utils::bool_to_byte,
};

/// The page size for each page.
/// By default, we will use 4kb as the size.
pub const PAGE_SIZE: usize = 4096;
pub const PTR_SIZE: usize = size_of::<usize>(); // 8 bytes on 64-bit systems

pub const IS_ROOT_SIZE: usize = 1;
pub const IS_ROOT_OFFSET: usize = 0;
pub const PARENT_POINTER_OFFSET: usize = 2;
pub const PARENT_POINTER_SIZE: usize = PTR_SIZE;
pub const NODE_TYPE_SIZE: usize = 1;
pub const NODE_TYPE_OFFSET: usize = 1;
pub const COMMON_NODE_HEADER_SIZE: usize = NODE_TYPE_SIZE + IS_ROOT_SIZE + PARENT_POINTER_SIZE;

pub const INTERNAL_NODE_NUM_CHILDREN_OFFSET: usize = COMMON_NODE_HEADER_SIZE;
pub const INTERNAL_NODE_NUM_CHILDREN_SIZE: usize = PTR_SIZE;
pub const INTERNAL_NODE_HEADER_SIZE: usize = COMMON_NODE_HEADER_SIZE + INTERNAL_NODE_NUM_CHILDREN_SIZE;

pub const LEAF_NODE_NUM_PAIRS_OFFSET: usize = COMMON_NODE_HEADER_SIZE;
pub const LEAF_NODE_NUM_PAIRS_SIZE: usize = PTR_SIZE;
pub const LEAF_NODE_HEADER_SIZE: usize = COMMON_NODE_HEADER_SIZE + LEAF_NODE_NUM_PAIRS_SIZE;

type PageData = [u8; PAGE_SIZE];

/// This is a wrapper for a value in a given page
pub struct Value(pub usize);

impl TryFrom<&[u8]> for Value {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() > PTR_SIZE {
            return Err(Error::TryFromSliceError(format!("Unexpected Error: array received is larger than the maximum allowed size ({}b)", PTR_SIZE)));
        }

        let mut truncated_arr = [0u8; PTR_SIZE];
        for (i, item) in value.iter().enumerate() {
            truncated_arr[i] = *item;
        }

        Ok(Value(usize::from_be_bytes(truncated_arr)))
    }
}

pub struct Page {
    data: Box<PageData>,
}

impl Page {
    pub fn new(data: PageData) -> Self {
        Self {
            data: Box::new(data),
        }
    }

    pub fn get_data(&self) -> PageData {
        *self.data
    }

    pub fn get_value_from_offset(&self, offset: usize) -> Result<usize, Error> {
        let bytes = &self.data[offset..offset + PTR_SIZE];
        let Value(res) = Value::try_from(bytes)?;
        Ok(res)
    }

    pub fn get_ptr_from_offset(&self, offset: usize, size: usize) -> &[u8] {
        &self.data[offset..offset + size]
    }
}

impl TryFrom<&Node> for Page {
    type Error = Error;

    fn try_from(node: &Node) -> Result<Self, Self::Error> {
        let mut data: PageData = [0x00; PAGE_SIZE];
        data[IS_ROOT_OFFSET] = bool_to_byte(node.is_root);
        data[NODE_TYPE_OFFSET] = u8::from(&node.node_type);

        if !node.is_root {
            let Some(Offset(parent_offset)) = node.parent_offset else {
                return Err(Error::UnexpectedError);
            };

            data[PARENT_POINTER_OFFSET..PARENT_POINTER_OFFSET + PARENT_POINTER_SIZE]
                .clone_from_slice(&parent_offset.to_be_bytes());
        }

        if !node.is_root {
        
        }

        match &node.node_type {
            NodeType::Internal(child_offsets, keys) => {
                data[INTERNAL_NODE_NUM_CHILDREN_OFFSET..INTERNAL_NODE_NUM_CHILDREN_OFFSET + INTERNAL_NODE_NUM_CHILDREN_SIZE]
                    .clone_from_slice(&child_offsets.len().to_be_bytes());

                let mut page_offset = INTERNAL_NODE_HEADER_SIZE;

                child_offsets.iter().for_each(|Offset(child_offset)| {
                    data[page_offset..page_offset + PTR_SIZE].clone_from_slice(&child_offset.to_be_bytes());
                    page_offset += PTR_SIZE;
                });

                for Key(key) in keys {
                    let key_bytes = key.as_bytes();
                    let mut raw_key: [u8; KEY_SIZE] = [0x00; KEY_SIZE];

                    if key_bytes.len() > KEY_SIZE {
                        return Err(Error::KeyOverflowError);
                    }

                    key_bytes.iter().enumerate().for_each(|(i, &byte)| {
                        raw_key[i] = byte;
                    });

                    data[page_offset..page_offset + KEY_SIZE].clone_from_slice(&raw_key);
                    page_offset += KEY_SIZE;
                }
            }
            NodeType::Leaf(key_value_pairs) => {
                data[LEAF_NODE_NUM_PAIRS_OFFSET..LEAF_NODE_NUM_PAIRS_OFFSET + LEAF_NODE_NUM_PAIRS_SIZE]
                    .clone_from_slice(&key_value_pairs.len().to_be_bytes());

                let mut page_offset = LEAF_NODE_HEADER_SIZE;
                for pair in key_value_pairs {
                    let key_bytes = pair.key.as_bytes();
                    let mut raw_key: [u8; KEY_SIZE] = [0x00; KEY_SIZE];

                    if key_bytes.len() > KEY_SIZE {
                        return Err(Error::KeyOverflowError);
                    }

                    key_bytes.iter().enumerate().for_each(|(i, &byte)| {
                        raw_key[i] = byte;
                    });

                    data[page_offset..page_offset + KEY_SIZE].clone_from_slice(&raw_key);
                    page_offset += KEY_SIZE;

                    let value_bytes = pair.value.as_bytes();
                    let mut raw_value: [u8; VALUE_SIZE] = [0x00; VALUE_SIZE];
                    
                    if value_bytes.len() > VALUE_SIZE {
                        return Err(Error::ValueOverflowError);
                    }

                    value_bytes.iter().enumerate().for_each(|(i, &byte)| {
                        raw_value[i] = byte;
                    });

                    data[page_offset..page_offset + VALUE_SIZE].clone_from_slice(&raw_value);
                    page_offset += VALUE_SIZE;
                }
            }
            NodeType::Unexpected => return Err(Error::UnexpectedError),
        }

        Ok(Self::new(data))
    }
}
