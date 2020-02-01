use std::io;
use std::cmp::min;
use std::convert::{TryFrom, TryInto};
use std::ops::Range;

use crate::{
  buf::{Buffer, BufCursor},
  scr::{Key, Position, Size, Window},
};

pub struct BufEditor {
  editor: BufCursor,
  anchor: BufCursor,
}

impl BufEditor {
  pub fn new() -> Self {
    BufEditor{editor: BufCursor::new(), anchor: BufCursor::new()}
  }
  pub fn handle_key(
    &mut self,
    buf: &mut Buffer,
    win: &Window,
    key: Key,
  ) {
    match key {
      Key::Left => self.editor.move_left(buf),
      Key::Right => self.editor.move_right(buf),
      Key::Up => self.editor.move_up(buf),
      Key::Down => self.editor.move_down(buf),
      Key::Char('\n') => {
        buf.break_line_at(&self.editor);
        self.editor.move_to_start_of_next_line(buf);
      }
      Key::Char(ch) => {
        buf.insert_at(ch, &self.editor);
        self.editor.move_right(buf); 
      }
      Key::Backspace => {
        if self.editor.col > 0 {
          buf.delete_before(&self.editor);
          self.editor.move_left(buf);
        } else if self.editor.row > 0 {
          self.editor.move_to_end_of_prev_line(buf);
          buf.merge_next_line_up(&self.editor);
        }
      }
      _ => (),
    }
    self.update_anchor(&win.size);
  }
  fn update_anchor(&mut self, size: &Size) {
    if self.editor.col < self.anchor.col {
      self.anchor.col = self.editor.col;
    }
    if self.editor.col > self.anchor.col + (size.cols as usize) - 1 {
      self.anchor.col = self.editor.col - (size.cols as usize) + 1;
    }
    if self.editor.row < self.anchor.row {
      self.anchor.row = self.editor.row;
    }
    if self.editor.row > self.anchor.row + (size.rows as usize) - 1 {
      self.anchor.row = self.editor.row - (size.rows as usize) + 1;
    }
  }
  pub fn draw(&self, buf: &Buffer, win: &mut Window) -> io::Result<()> {
    for row in self.line_range(buf, &win.size) {
      let range = self.char_range(buf, &win.size, row);
      let start = range.start;
      win.put_at(
        buf.line(row).get(range).unwrap(),
        Position{row: row.try_into().unwrap(), col: start.try_into().unwrap()}
      )?;
    }
    Ok(())
  }
  fn char_range(&self, buf: &Buffer, size: &Size, row: usize) -> Range<usize> {
    let start = self.anchor.col;
    let len = min(buf.width(row) - start, size.cols as usize);
    start..(start + len)
  }
  fn line_range(&self, buf: &Buffer, size: &Size) -> Range<usize> {
    let start = self.anchor.row;
    let len = min(buf.height() - start, size.rows as usize);
    start..(start + len)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::scr::{tests::*, WindowManager};

  #[test]
  fn test_editor() {
    let mut ed = BufEditor::new();
    let mut buf = Buffer::try_from("abcd\nefgh\nijkl\nmnop").unwrap();
    let mut scr = TestScreen::new(Vec::new());
    {
      let mut wm = WindowManager::new(&mut scr);
      let wid = wm.create_full().unwrap();
      let mut win = wm.borrow_mut(wid).unwrap();
      ed.draw(&buf, &mut win).unwrap();
    }
  }

  // empty buffer
  // square buffer
  // irregular buffer
  // move up,down,left,right with,without screen move
  // past top,bottom-left,right
  // past end,start-of-line
  // insert,backspace,break-line with,without screen move
}
