use std::path::Path;

use crate::{
  error::Error, 
  node::{Node, NodeType},
  pager::Pager,
  page::Page,
};

const MAX_BRANCHING_FACTOR: usize = 200;
const NODE_KEYS_LIMIT: usize = MAX_BRANCHING_FACTOR - 1;

#[derive(Debug)]
pub struct BTree {
  path: &'static Path,
  branches: usize,
  pager: Pager,
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

    Ok(Self {
      pager,
      path,
      branches,
    })
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
