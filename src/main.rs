/*
[x] Create a new file and open it
[x] Open an existing file
[x] Navigate to a location in an open file
[x] Insert a character at the current location
[x] Delete a character at the current location
[x] Write an open file out
*/

extern crate termion;

use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::ops::Range;

use termion::{
  raw::IntoRawMode,
  input::TermRead,
};

type Line = String;
type Buffer = Vec<Line>;

fn open_file(path: &str) -> io::Result<fs::File> {
  fs::OpenOptions::new().read(true).write(true).create(true).open(path)
}

fn read_file(path: &str) -> io::Result<Buffer> {
  let file = open_file(path)?;
  BufReader::new(file).lines().collect()
}

fn write_file(path: &str, buf: &Buffer) -> io::Result<()> {
  let mut file = open_file(path)?;
  for line in buf {
    writeln!(file, "{}", line)?;
  }
  file.flush()
}

struct Cursor {
  col: usize,
  row: usize,
  left: usize,
  top: usize,
}

impl Cursor {
  fn new() -> Self {
    Cursor{col: 0, row: 0, left: 0, top: 0}
  }
}

fn insert_at(ch: char, cur: &mut Cursor, buf: &mut Buffer) {
  buf[cur.row].insert(cur.col, ch)
}

fn delete_at(cur: &mut Cursor, buf: &mut Buffer) {
  buf[cur.row].remove(cur.col);
}

type Screen = termion::raw::RawTerminal<io::Stdout>;

struct Size {
  rows: usize,
  cols: usize,
}

impl Size {
  fn new<T: Into<usize>>(rows: T, cols: T) -> Self {
    Size{rows: rows.into(), cols: cols.into()}
  }
}

fn buffer_char_range(cur: &Cursor, size: &Size) -> Range<usize> {
  cur.left..(cur.left + size.cols)
}

fn write_line_to_screen(
  scr: &mut Screen,
  cur: &Cursor,
  line: &Line,
  size: &Size,
) -> io::Result<()> {
  let bytes = line.as_bytes();
  for i in buffer_char_range(cur, size) {
    if i >= line.len() {
      break;
    }
    write!(scr, "{}", bytes[i] as char)?;
  }
  write!(scr, "\n\r")?;
  Ok(())
}

fn buffer_line_range(cur: &Cursor, size: &Size) -> Range<usize> {
  cur.top..(cur.top + size.rows)
}

fn cursor_screen_position(cur: &Cursor) -> (u16, u16) {
  ((cur.row - cur.top + 1) as u16, (cur.col - cur.left + 1) as u16)
}

fn write_buffer_to_screen(
  scr: &mut Screen,
  cur: &Cursor,
  buf: &Buffer,
  size: &Size,
) -> io::Result<()> {
  for i in buffer_line_range(cur, size) {
    write_line_to_screen(scr, cur, &buf[i], size)?;
  }
  let (r, c) = cursor_screen_position(cur);
  write!(scr, "{}", termion::cursor::Goto(c, r))?;
  scr.flush()
}

fn clear_screen(scr: &mut Screen) -> io::Result<()> {
  write!(scr, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1))?;
  scr.flush()
}

fn update_screen(
  scr: &mut Screen,
  cur: &Cursor,
  buf: &Buffer,
  size: &Size,
) -> io::Result<()> {
  clear_screen(scr)?;
  write_buffer_to_screen(scr, cur, buf, size)
}

fn init_screen() -> io::Result<Screen> {
  io::stdout().into_raw_mode()
}

fn align_cursor(cur: &mut Cursor, size: &Size) {
  if cur.col < cur.left {
    cur.left = cur.col;
  }
  if cur.col > cur.left + size.cols - 1 {
    cur.left = cur.col - size.cols + 1;
  }
  if cur.row < cur.top {
    cur.top = cur.row;
  }
  if cur.row > cur.top + size.rows - 1 {
    cur.top = cur.row - size.rows + 1;
  }
}

fn move_cursor_left(cur: &mut Cursor, _buf: &Buffer, size: &Size) {
  if cur.col > 0 {
    cur.col -= 1;
  }
  align_cursor(cur, size);
}

fn move_cursor_right(cur: &mut Cursor, buf: &Buffer, size: &Size) {
  if cur.col + 1 < buf[cur.row].len() {
    cur.col += 1;
  }
  align_cursor(cur, size);
}

fn truncate_cursor_to_line(cur: &mut Cursor, buf: &Buffer, size: &Size) {
  if buf[cur.row].len() == 0 {
    cur.col = 0;
  } else if cur.col >= buf[cur.row].len() {
    cur.col = buf[cur.row].len() - 1;
  }
  align_cursor(cur, size);
}

fn move_cursor_up(cur: &mut Cursor, buf: &Buffer, size: &Size) {
  if cur.row > 0 {
    cur.row -= 1;
  }
  truncate_cursor_to_line(cur, buf, size);
  align_cursor(cur, size);
}

fn move_cursor_down(cur: &mut Cursor, buf: &Buffer, size: &Size) {
  if cur.row + 1 < buf.len() {
    cur.row += 1;
  }
  truncate_cursor_to_line(cur, buf, size);
  align_cursor(cur, size);
}

type Key = termion::event::Key;

fn get_screen_size() -> io::Result<Size> {
  termion::terminal_size().map(|(cols, rows)| Size::new(rows, cols))
}

fn edit_buffer(buf: &mut Buffer) -> io::Result<()> {
  let mut scr = init_screen()?;
  let mut cur = Cursor::new();
  let mut size = get_screen_size()?;
  update_screen(&mut scr, &cur, buf, &size)?;
  for res in io::stdin().keys() {
    let key = res?;
    size = get_screen_size()?;
    match key {
      Key::Left => move_cursor_left(&mut cur, buf, &size),
      Key::Right => move_cursor_right(&mut cur, buf, &size),
      Key::Up => move_cursor_up(&mut cur, buf, &size),
      Key::Down => move_cursor_down(&mut cur, buf, &size),
      Key::Char(ch) => insert_at(ch, &mut cur, buf),
      Key::Backspace => delete_at(&mut cur, buf),
      _ => break,
    }
    update_screen(&mut scr, &cur, buf, &size)?;
  }
  Ok(())
}

fn main() -> io::Result<()> {
  match env::args().skip(1).next() {
    Some(path) => {
      let mut buf = read_file(&path)?;
      edit_buffer(&mut buf)?;
      write_file(&path, &buf)
    }
    None => Ok(()),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn check_range(
    cur: &Cursor,
    size: &Size,
    l: Range<usize>,
    c: Range<usize>,
  ) {
    assert_eq!(l, buffer_line_range(cur, size));
    assert_eq!(c, buffer_char_range(cur, size));
  }

  fn apply_and_check(
    cur: &mut Cursor,
    buf: &Buffer,
    size: &Size,
    f: fn(cur: &mut Cursor, buf: &Buffer, size: &Size),
    l: Range<usize>,
    c: Range<usize>,
  ) {
    f(cur, buf, size);
    check_range(cur, size, l, c);
  }

  #[test]
  fn test_size() {
    let size = get_screen_size().unwrap();
    assert!(size.cols > size.rows);
  }

  #[test]
  fn test_cursor() {
    let buf: Buffer = vec![
      "1234".into(),
      "2345".into(),
      "3456".into(),
      "4567".into(),
      "5678".into(),
    ];
    let size = Size::new(3usize, 2usize);
    let mut cur = Cursor::new();

    check_range(&cur, &size, 0..3, 0..2);
    apply_and_check(&mut cur, &buf, &size, move_cursor_left, 0..3, 0..2);
    apply_and_check(&mut cur, &buf, &size, move_cursor_up, 0..3, 0..2);
    apply_and_check(&mut cur, &buf, &size, move_cursor_right, 0..3, 0..2);
    apply_and_check(&mut cur, &buf, &size, move_cursor_right, 0..3, 1..3);
    apply_and_check(&mut cur, &buf, &size, move_cursor_right, 0..3, 2..4);
    apply_and_check(&mut cur, &buf, &size, move_cursor_right, 0..3, 2..4);
    apply_and_check(&mut cur, &buf, &size, move_cursor_down, 0..3, 2..4);
    apply_and_check(&mut cur, &buf, &size, move_cursor_down, 0..3, 2..4);
    apply_and_check(&mut cur, &buf, &size, move_cursor_down, 1..4, 2..4);
    apply_and_check(&mut cur, &buf, &size, move_cursor_down, 2..5, 2..4);
    apply_and_check(&mut cur, &buf, &size, move_cursor_down, 2..5, 2..4);
    apply_and_check(&mut cur, &buf, &size, move_cursor_left, 2..5, 2..4);
    apply_and_check(&mut cur, &buf, &size, move_cursor_left, 2..5, 1..3);
    apply_and_check(&mut cur, &buf, &size, move_cursor_left, 2..5, 0..2);
    apply_and_check(&mut cur, &buf, &size, move_cursor_up, 2..5, 0..2);
    apply_and_check(&mut cur, &buf, &size, move_cursor_up, 2..5, 0..2);
    apply_and_check(&mut cur, &buf, &size, move_cursor_up, 1..4, 0..2);
    apply_and_check(&mut cur, &buf, &size, move_cursor_up, 0..3, 0..2);
    apply_and_check(&mut cur, &buf, &size, move_cursor_up, 0..3, 0..2);
  }
}
