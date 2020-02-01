use std::convert::TryFrom;
use std::io::{self, BufRead, BufReader, Read};

/// BufCursor is a cursor into a buffer
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BufCursor {
  pub col: usize,
  pub row: usize,
}

impl BufCursor {
  pub fn new() -> Self {
    BufCursor{col: 0, row: 0}
  }

  pub fn at(row: usize, col: usize) -> Self {
    BufCursor{col, row}
  }

  pub fn move_right(&mut self, buf: &Buffer) {
    if self.row < buf.height() {
      if self.col < buf.width(self.row) {
        self.col += 1;
      } else {
        self.row += 1;
        self.col = 0;
      }
    } 
  }

  pub fn move_left(&mut self, buf: &Buffer) {
    if self.col > 0 {
      self.col -= 1;
    } else if self.row > 0 {
      self.row -= 1;
      self.col = buf.width(self.row);
    }
  }

  pub fn move_up(&mut self, buf: &Buffer) {
    if self.row > 0 {
      self.row -= 1;
    }
    self.trim_cursor_to_end_of_line(buf);
  }

  pub fn move_down(&mut self, buf: &Buffer) {
    if self.row < buf.height() {
      self.row += 1;
    }
    self.trim_cursor_to_end_of_line(buf);
  }

  fn trim_cursor_to_end_of_line(&mut self, buf: &Buffer) {
    let width = buf.width(self.row);
    if self.col > width {
      self.col = width;
    }
  }

  pub fn move_to_start_of_line(&mut self, _buf: &Buffer) {
    self.col = 0;
  }

  pub fn move_to_end_of_line(&mut self, buf: &Buffer) {
    self.col = buf.width(self.row);
  }

  pub fn move_to_end_of_prev_line(&mut self, buf: &Buffer) {
    self.move_up(buf);
    self.move_to_end_of_line(buf);
  }

  pub fn move_to_start_of_next_line(&mut self, buf: &Buffer) {
    self.move_down(buf);
    self.move_to_start_of_line(buf);
  }
}

type Line = String;

/// Buffer is the core type containing lines of characters for editing
pub struct Buffer {
  lines: Vec<Line>,
}

impl From<Vec<Line>> for Buffer {
  fn from(v: Vec<Line>) -> Self {
    Buffer{ lines: v }
  }
}

impl<R: Read> TryFrom<BufReader<R>> for Buffer {
  type Error = io::Error;

  fn try_from(b: BufReader<R>) -> Result<Self, Self::Error> {
    Ok(Buffer::from(b.lines().collect::<Result<Vec<Line>, Self::Error>>()?))
  }
}

impl TryFrom<&str> for Buffer {
  type Error = io::Error;

  fn try_from(s: &str) -> Result<Self, Self::Error> {
    Buffer::try_from(BufReader::new(s.as_bytes()))
  }
}


impl Buffer {
  pub fn new() -> Self {
    Buffer{ lines: Vec::new() }
  }

  pub fn line(&self, row: usize) -> &Line {
    &self.lines[row]
  }

  pub fn height(&self) -> usize {
    self.lines.len()
  }

  pub fn width(&self, row: usize) -> usize {
    if row == self.height() {
      return 0;
    }
    if row > self.height() {
      panic!("tried to get width past last row of buffer");
    }
    self.lines[row].len()
  }

  pub fn char(&self, cur: &BufCursor) -> char {
    if cur.row >= self.height() {
      panic!("tried to get char past last row of buffer");
    }
    if cur.col >= self.width(cur.row) {
      panic!("tried to get char past last char of line");
    }
    self.lines[cur.row].as_bytes()[cur.col] as char
  }

  // insert a character into the buffer under the cursor
  //
  // inserting one line past the end of the buffer adds a new line first.
  // inserting one column past the end of the line adds a new character.
  pub fn insert_at(&mut self, ch: char, cur: &BufCursor) {
    if cur.row > self.height() {
      panic!("tried to insert past last line of buffer");
    }
    if cur.col > self.width(cur.row) {
      panic!("tried to insert past end of line");
    }
    if cur.row == self.height() {
      self.lines.push(Line::new());
    }
    self.lines[cur.row].insert(cur.col, ch)
  }

  // delete a character from the buffer just before the cursor
  pub fn delete_before(&mut self, cur: &BufCursor) {
    if cur.col == 0 {
      panic!("tried to delete before beginning of buffer");
    }
    if cur.row >= self.height() {
      panic!("tried to delete past last row of buffer");
    }
    if cur.col > self.width(cur.row) {
      panic!("tried to delete past end of line");
    }
    self.lines[cur.row].remove(cur.col - 1);
  }

  // merge the line after this line with this line
  pub fn merge_next_line_up(&mut self, cur: &BufCursor) {
    if cur.row + 1 >= self.height() {
      panic!("tried to merge line from past end of buffer");
    }
    let line = self.lines.remove(cur.row + 1);
    self.lines[cur.row].push_str(&line);
  }

  // create a new line and move the rest of this line to it
  pub fn break_line_at(&mut self, cur: &BufCursor) {
    if cur.row > self.height() {
      panic!("tried to break line past last row of buffer");
    }
    if cur.col > self.width(cur.row) {
      panic!("tried to break line past end of line");
    }
    if cur.row == self.height() {
      self.lines.push(Line::new());
      return;
    }
    let new_line = self.lines[cur.row].split_off(cur.col);
    self.lines.insert(cur.row + 1, new_line);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use crate::tests::assert_panics;

  #[test]
  fn test_new_cursor_position() {
    let cur = BufCursor::new();
    assert_eq!(0, cur.row);
    assert_eq!(0, cur.col);
  }

  fn check_cursor(cur: &BufCursor, row: usize, col: usize) {
    assert_eq!(cur, &BufCursor::at(row, col))
  }

  #[test]
  fn test_move_cursor() {
    let mut cur = BufCursor::new();
    let buf = Buffer::try_from("123\n45\n678").unwrap();


    // moving left at start does nothing
    cur.move_left(&buf);
    check_cursor(&cur, 0, 0);
    
    // moving up at start does nothing
    cur.move_up(&buf);
    check_cursor(&cur, 0, 0);

    // moving right works
    cur.move_right(&buf);
    check_cursor(&cur, 0, 1);

    // moving down works
    cur.move_down(&buf);
    check_cursor(&cur, 1, 1);

    // moving left works
    cur.move_left(&buf);
    check_cursor(&cur, 1, 0);

    // moving up works
    cur.move_up(&buf);
    check_cursor(&cur, 0, 0);

    // moving past end of line wraps around
    cur.move_right(&buf);
    cur.move_right(&buf);
    cur.move_right(&buf);
    cur.move_right(&buf);
    check_cursor(&cur, 1, 0);

    // moving before start of line wraps back
    cur.move_left(&buf);
    check_cursor(&cur, 0, 3);

    // moving down to shorter line trims the cursor
    cur.move_down(&buf);
    check_cursor(&cur, 1, 2);

    // moving down to a longer line does not trim the cursor
    cur.move_down(&buf);
    check_cursor(&cur, 2, 2);

    // moving up to a shorter line trims the cursor
    cur.move_right(&buf);
    cur.move_up(&buf);
    check_cursor(&cur, 1, 2);

    // moving up to a longer line does not trim the cursor
    cur.move_up(&buf);
    check_cursor(&cur, 0, 2);

    // moving past end of the buffer works
    cur.move_down(&buf);
    cur.move_down(&buf);
    cur.move_down(&buf);
    check_cursor(&cur, 3, 0);

    // moving down or right at end of the buffer does nothing
    cur.move_down(&buf);
    check_cursor(&cur, 3, 0);
    cur.move_right(&buf);
    check_cursor(&cur, 3, 0);

    // moving left at end of buffer wraps around
    cur.move_left(&buf);
    check_cursor(&cur, 2, 3);

    // moving right at last filled line of buffer wraps around
    cur.move_right(&buf);
    check_cursor(&cur, 3, 0);

    // moving to start, end of line at end of buffer works
    cur.move_to_start_of_line(&buf);
    check_cursor(&cur, 3, 0);
    cur.move_to_end_of_line(&buf);
    check_cursor(&cur, 3, 0);

    // moving to start of next line at end of buffer does nothing
    cur.move_to_start_of_next_line(&buf);

    // moving to end of prev line works
    cur.move_to_end_of_prev_line(&buf);
    check_cursor(&cur, 2, 3);
    cur.move_to_end_of_prev_line(&buf);
    check_cursor(&cur, 1, 2);
    cur.move_to_end_of_prev_line(&buf);
    check_cursor(&cur, 0, 3);

    // moving to end of prev line at beginning of buffer does nothing
    cur.move_to_end_of_prev_line(&buf);
    check_cursor(&cur, 0, 3);

    // moving to start of next line works
    cur.move_to_start_of_next_line(&buf);
    check_cursor(&cur, 1, 0);
    cur.move_to_start_of_next_line(&buf);
    check_cursor(&cur, 2, 0);
    cur.move_to_start_of_next_line(&buf);
    check_cursor(&cur, 3, 0);

    // moving to start, end of line works
    cur.move_up(&buf);
    cur.move_to_start_of_line(&buf);
    check_cursor(&cur, 2, 0);
    cur.move_to_end_of_line(&buf);
    check_cursor(&cur, 2, 3);
  }

  #[test]
  fn test_new_buffer_empty() {
    let buf = Buffer::new();
    let cur = BufCursor::new();
    assert_eq!(0, buf.height());
    assert_eq!(0, buf.width(cur.row));
  }

  #[test]
  fn test_insert_at() {
    // inserting two lines past end of the buffer panics
    assert_panics(|| {
      let mut buf = Buffer::new();
      let mut cur = BufCursor::new();
      cur.row = 1;
      buf.insert_at('a', &cur);
    });

    // inserting two char past end of the line panics
    assert_panics(|| {
      let mut buf = Buffer::new();
      let mut cur = BufCursor::new();
      buf.insert_at('a', &cur);
      cur.col = 2;
      buf.insert_at('b', &cur);
    });

    // inserting one char past start of new line panics
    assert_panics(|| {
      let mut buf = Buffer::new();
      let mut cur = BufCursor::new();
      cur.col = 1;
      buf.insert_at('a', &cur);
    });

    let mut buf = Buffer::new();
    let mut cur = BufCursor::new();

    // inserting one line past end of the buffer adds a new line
    assert_eq!(0, buf.height());
    buf.insert_at('a', &cur);
    assert_eq!(1, buf.height());
    assert_eq!(1, buf.width(cur.row));
    assert_eq!('a', buf.char(&cur));

    // inserting one char past end of the line adds a new char
    cur.col = 1;
    buf.insert_at('b', &cur);
    assert_eq!(1, buf.height());
    assert_eq!(2, buf.width(cur.row));
    assert_eq!('b', buf.char(&cur));

    // inserting more characters works
    buf.insert_at('c', &cur);
    buf.insert_at('d', &cur);
    buf.insert_at('e', &cur);
    assert_eq!('e', buf.char(&cur));
    cur.col += 1;
    assert_eq!('d', buf.char(&cur));
    cur.col += 1;
    assert_eq!('c', buf.char(&cur));
    cur.col += 1;
    assert_eq!('b', buf.char(&cur));
  }

  #[test]
  fn test_delete_before() {
    // deleting at the beginning of a buffer panics
    assert_panics(|| {
      let mut buf = Buffer::new();
      let cur = BufCursor::new();
      buf.delete_before(&cur);
    });

    // deleting at the beginning of a line panics
    assert_panics(|| {
      let mut buf = Buffer::new();
      let cur = BufCursor::new();
      buf.insert_at('a', &cur);
      buf.delete_before(&cur);
    });

    // deleting from the end of a line works
    let mut buf = Buffer::new();
    let mut cur = BufCursor::new();
    buf.insert_at('a', &cur);
    assert_eq!(1, buf.width(cur.row));
    cur.col = 1;
    buf.delete_before(&cur);
    assert_eq!(0, buf.width(cur.row));

    // deleting more characters works
    cur.col = 0;
    buf.insert_at('b', &cur);
    buf.insert_at('c', &cur);
    buf.insert_at('d', &cur);
    assert_eq!(3, buf.width(cur.row));
    cur.col = 1;
    buf.delete_before(&cur);
    assert_eq!('b', buf.char(&cur));
    buf.delete_before(&cur);
    buf.delete_before(&cur);
    assert_eq!(0, buf.width(cur.row));
  }

  #[test]
  fn test_merge_next_line_up() {
    // merging past end of buffer panics
    assert_panics(|| {
      let mut buf = Buffer::new();
      let cur = BufCursor::new();
      buf.merge_next_line_up(&cur);
    });

    assert_panics(|| {
      let mut buf = Buffer::new();
      let cur = BufCursor::new();
      buf.insert_at('a', &cur);
      buf.merge_next_line_up(&cur);
    });

    // merging works
    let mut buf = Buffer::new();
    let mut cur = BufCursor::new();
    buf.insert_at('a', &cur);
    cur.row = 1;
    buf.insert_at('b', &cur);
    cur.row = 0;
    buf.merge_next_line_up(&cur);
    assert_eq!(2, buf.width(cur.row));
    assert_eq!('a', buf.char(&cur));
    cur.col = 1;
    assert_eq!('b', buf.char(&cur));
  }

  #[test]
  fn test_break_line_at() {
    // breaking past end of buffer panics
    assert_panics(|| {
      let mut buf = Buffer::new();
      let mut cur = BufCursor::new();
      cur.row = 1;
      buf.break_line_at(&cur);
    });

    // breaking past end of line panics
    assert_panics(|| {
      let mut buf = Buffer::new();
      let mut cur = BufCursor::new();
      buf.insert_at('a', &cur);
      cur.col = 2;
      buf.break_line_at(&cur);
    });

    // breaking one char past start of new line panics
    assert_panics(|| {
      let mut buf = Buffer::new();
      let mut cur = BufCursor::new();
      cur.col = 1;
      buf.break_line_at(&cur);
    });

    // breaking empty lines works
    let mut buf = Buffer::new();
    let cur = BufCursor::new();
    assert_eq!(0, buf.height());
    assert_eq!(0, buf.width(cur.row));
    buf.break_line_at(&cur);
    assert_eq!(1, buf.height());
    assert_eq!(0, buf.width(cur.row));
    buf.break_line_at(&cur);
    assert_eq!(2, buf.height());
    assert_eq!(0, buf.width(cur.row));

    // breaking at end of buffer works
    let mut buf = Buffer::new();
    let mut cur = BufCursor::new();
    buf.insert_at('a', &cur);
    buf.insert_at('b', &cur);
    cur.col = 1;
    buf.break_line_at(&cur);
    assert_eq!(2, buf.height());
    cur.col = 0;
    assert_eq!(1, buf.width(cur.row));
    assert_eq!('b', buf.char(&cur));
    cur.row = 1;
    assert_eq!(1, buf.width(cur.row));
    assert_eq!('a', buf.char(&cur));

    // breaking at end of non-empty line works
    cur.row = 0;
    buf.merge_next_line_up(&cur);
    cur.col = 2;
    buf.break_line_at(&cur);
    assert_eq!(2, buf.height());
    assert_eq!(2, buf.width(cur.row));
    cur.row = 1;
    assert_eq!(0, buf.width(cur.row));
  }
}
