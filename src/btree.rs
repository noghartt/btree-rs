use std::path::Path;

use crate::{
  error::Error,
  node::{Key, KeyValuePair, Node, NodeType, Offset},
  page::Page,
  pager::Pager, wal::Wal
};

const MAX_BRANCHING_FACTOR: usize = 200;
const NODE_KEYS_LIMIT: usize = MAX_BRANCHING_FACTOR - 1;

#[derive(Debug)]
pub struct BTree {
  path: &'static Path,
  branches: usize,
  pager: Pager,
  wal: Wal,
}

impl BTree {
  pub fn new(path: &'static Path, branches: usize) -> Result<Self, Error> {
    if branches == 0 || branches > MAX_BRANCHING_FACTOR {
      return Err(Error::UnexpectedError);
    }

    let mut pager = Pager::new(path)?;
    let root = Node::new(NodeType::Leaf(vec![]), true, None);
    let root_offset = pager.write_page(Page::try_from(&root)?)?;
    let parent_directory = path.parent().unwrap_or_else(|| Path::new("/tmp"));
    let mut wal = Wal::new(parent_directory.to_path_buf())?;
    wal.set_root(root_offset)?;

    Ok(Self {
      pager,
      path,
      branches,
      wal,
    })
  }

  pub fn insert(&mut self, kv: KeyValuePair) -> Result<(), Error> {
    let root_offset = self.wal.get_root()?;
    let root_page = self.pager.get_page(&root_offset)?;
    let mut new_root_offset: Offset;
    let mut new_root: Node;

    let mut root = Node::try_from(root_page)?;
    if self.is_node_full(&root)? {
        new_root = Node::new(NodeType::Internal(vec![], vec![]), true, None);
        new_root_offset = self.pager.write_page(Page::try_from(&new_root)?)?;
        root.parent_offset = Some(new_root_offset.clone());
        root.is_root = false;
        let (median, sibling) = root.split(self.branches)?;
        let old_root_offset = self.pager.write_page(Page::try_from(&root)?)?;
        let sibling_offset = self.pager.write_page(Page::try_from(&sibling)?)?;
        new_root.node_type = NodeType::Internal(vec![old_root_offset, sibling_offset], vec![median]);
        self.pager.write_page_at_offset(Page::try_from(&new_root)?, &new_root_offset)?;
    } else {
        new_root = root.clone();
        new_root_offset = self.pager.write_page(Page::try_from(&new_root)?)?;
    }

    self.insert_non_full(&mut new_root, new_root_offset.clone(), kv)?;
    self.wal.set_root(new_root_offset)
  }

  fn insert_non_full(&mut self, node: &mut Node, node_offset: Offset, kv: KeyValuePair) -> Result<(), Error> {
    match &mut node.node_type {
        NodeType::Leaf(ref mut pairs) => {
            let idx = pairs.binary_search(&kv).unwrap_or_else(|x| x);
            pairs.insert(idx, kv);
            self.pager.write_page_at_offset(Page::try_from(&*node)?, &node_offset)
        }
        NodeType::Internal(ref mut children, ref mut keys) => {
            let idx = keys.binary_search(&Key(kv.key.clone())).unwrap_or_else(|x| x);
            let child_offset = children.get(idx).ok_or(Error::UnexpectedError)?.clone();
            let child_page = self.pager.get_page(&child_offset)?;
            let mut child = Node::try_from(child_page)?;
            let new_child_offset = self.pager.write_page(Page::try_from(&child)?)?;
            children[idx] = new_child_offset.to_owned();
            if self.is_node_full(&child)? {
                let (median, mut sibling) = child.split(self.branches)?;
                self.pager.write_page_at_offset(Page::try_from(&child)?, &new_child_offset)?;
                let sibling_offset = self.pager.write_page(Page::try_from(&sibling)?)?;
                children.insert(idx + 1, sibling_offset.clone());
                keys.insert(idx, median.clone());
                self.pager.write_page_at_offset(Page::try_from(&*node)?, &node_offset)?;
                if kv.key <= median.0 {
                    self.insert_non_full(&mut child, new_child_offset, kv)
                } else {
                    self.insert_non_full(&mut sibling, sibling_offset, kv)
                }
            } else {
                self.pager.write_page_at_offset(Page::try_from(&*node)?, &node_offset)?;
                self.insert_non_full(&mut child, new_child_offset, kv)
            }
        }
        NodeType::Unexpected => Err(Error::UnexpectedError),
    }
  }

  fn is_node_full(&self, node: &Node) -> Result<bool, Error> {
    match &node.node_type {
      NodeType::Leaf(pairs) => Ok(pairs.len() == (2 * self.branches - 1)),
      NodeType::Internal(_, keys) => Ok(keys.len() == (2 * self.branches - 1)),
      NodeType::Unexpected => Err(Error::UnexpectedError)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_create_new_btree() {
    let path = Path::new("/tmp/db");
    let branches = 10;

    let btree = BTree::new(path, branches).unwrap();

    assert_eq!(btree.branches, branches);
    assert_eq!(btree.path, path);
  }
}
