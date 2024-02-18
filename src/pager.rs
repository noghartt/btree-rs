use std::{
  fs::{File, OpenOptions},
  io::{Seek, SeekFrom, Write},
  path::Path,
};

use crate::{
  error::Error,
  page::{Page, PAGE_SIZE},
  node::Offset,
};

#[derive(Debug)]
pub struct Pager {
  file: File,
  cursor: usize,
}

impl Pager {
  pub fn new(path: &Path) -> Result<Self, Error> {
    let fd = OpenOptions::new()
      .create(true)
      .read(true)
      .write(true)
      .truncate(true)
      .open(path)?;

    Ok(Self {
      file: fd,
      cursor: 0,
    })
  }

  pub fn write_page(&mut self, page: Page) -> Result<Offset, Error> {
    self.file.seek(SeekFrom::Start(self.cursor as u64))?;
    self.file.write_all(&page.get_data())?;
    let res = Offset(self.cursor);
    self.cursor += PAGE_SIZE;
    Ok(res)
  }
}
