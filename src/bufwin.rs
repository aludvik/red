use std::io;
use std::cmp::min;
use std::convert::TryInto;
use std::ops::Range;

use crate::{
  buf::{Buffer, BufCursor},
  scr::{Position, Screen, Window},
};

pub struct BufEditor {
  editor: BufCursor,
  anchor: BufCursor,
}

impl BufEditor {
  pub fn fill_with_buffer(
    &self,
    buf: &Buffer,
    win: &mut Window,
  ) -> io::Result<()> {
    for row in self.line_range(buf) {
      let range = self.char_range(buf, row);
      let start = range.start;
      win.put_at(
        buf.line(row).get(range).unwrap(),
        Position{row: row.try_into().unwrap(), col: start.try_into().unwrap()}
      )?;
    }
    Ok(())
  }

  fn char_range(&self, buf: &Buffer, win: &Window, row: usize) -> Range<usize> {
    let start = self.anchor.col;
    let len = min(buf.width(row) - start, win.size.cols as usize);
    start..(start + len)
  }
  
  fn line_range(&self, buf: &Buffer, win: &Window) -> Range<usize> {
    let start = self.anchor.row;
    let len = min(buf.height() - start, win.size.rows as usize);
    start..(start + len)
  }
}


