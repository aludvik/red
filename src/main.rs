#[cfg(test)]
extern crate tempfile;
extern crate termion;

#[cfg(test)]
mod tests;

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
type Screen = termion::raw::RawTerminal<io::Stdout>;
type Key = termion::event::Key;

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

struct Size {
  rows: usize,
  cols: usize,
}

impl Size {
  fn new<T: Into<usize>>(rows: T, cols: T) -> Self {
    Size{rows: rows.into(), cols: cols.into()}
  }
}

fn get_screen_size() -> io::Result<Size> {
  termion::terminal_size().map(|(cols, rows)| Size::new(rows, cols))
}

// file system functions
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

// buffer mutations
fn init_buffer_if_empty(buf: &mut Buffer) {
  if buf.len() == 0 {
    buf.push(String::new());
  }
}

fn insert_at(ch: char, cur: &Cursor, buf: &mut Buffer) {
  if cur.row == buf.len() {
    buf.push(Line::new());
  }
  buf[cur.row].insert(cur.col, ch)
}

fn delete_at(cur: &Cursor, buf: &mut Buffer) {
  buf[cur.row].remove(cur.col - 1);
}

fn merge_into_above(cur: &Cursor, buf: &mut Buffer) {
  if cur.row > 0 && cur.row < buf.len() {
    let line = buf.remove(cur.row);
    buf[cur.row - 1].push_str(&line);
  }
}

fn break_line_at(cur: &Cursor, buf: &mut Buffer) {
  let new_line = buf[cur.row].split_off(cur.col);
  buf.insert(cur.row+1, new_line);
}

// screen updating
fn buffer_char_range(cur: &Cursor, size: &Size) -> Range<usize> {
  cur.left..(cur.left + size.cols)
}

fn buffer_line_range(cur: &Cursor, size: &Size) -> Range<usize> {
  cur.top..(cur.top + size.rows)
}

fn cursor_screen_position(cur: &Cursor) -> (u16, u16) {
  ((cur.row - cur.top + 1) as u16, (cur.col - cur.left + 1) as u16)
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
  Ok(())
}

fn write_buffer_to_screen(
  scr: &mut Screen,
  cur: &Cursor,
  buf: &Buffer,
  size: &Size,
) -> io::Result<()> {
  let range = buffer_line_range(cur, size);
  let last = range.end - 1;
  for i in range {
    if i >= buf.len() {
      break;
    }
    write_line_to_screen(scr, cur, &buf[i], size)?;
    if i != last {
      write!(scr, "\n\r")?;
    }
  }
  let (r, c) = cursor_screen_position(cur);
  write!(scr, "{}", termion::cursor::Goto(c, r))
}

fn clear_screen(scr: &mut Screen) -> io::Result<()> {
  write!(scr, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1))
}

fn init_screen() -> io::Result<Screen> {
  io::stdout().into_raw_mode()
}

fn update_screen(
  scr: &mut Screen,
  cur: &Cursor,
  buf: &Buffer,
  size: &Size,
) -> io::Result<()> {
  clear_screen(scr)?;
  write_buffer_to_screen(scr, cur, buf, size)?;
  scr.flush()
}

// Cursor movement
fn align_cursor(cur: &mut Cursor, size: &Size) {
  if cur.col < cur.left {
    cur.left = cur.col;
  }
  if cur.col > cur.left + size.cols {
    cur.left = cur.col - size.cols;
  }
  if cur.row < cur.top {
    cur.top = cur.row;
  }
  if cur.row > cur.top + size.rows {
    cur.top = cur.row - size.rows;
  }
}

fn move_cursor_left(cur: &mut Cursor, buf: &Buffer, size: &Size) {
  if cur.col > 0 {
    cur.col -= 1;
  } else if cur.row > 0 {
    cur.row -= 1;
    cur.col = buf[cur.row].len();
  }
  align_cursor(cur, size);
}

fn move_cursor_right(cur: &mut Cursor, buf: &Buffer, size: &Size) {
  if cur.row < buf.len() {
    if cur.col < buf[cur.row].len() {
      cur.col += 1;
    } else {
      cur.row += 1;
      cur.col = 0;
    }
  }
  align_cursor(cur, size);
}

fn truncate_cursor_to_line(cur: &mut Cursor, buf: &Buffer, size: &Size) {
  if cur.row < buf.len() {
    if cur.col > buf[cur.row].len() {
      cur.col = buf[cur.row].len();
    }
  } else {
    cur.col = 0;
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
  if cur.row < buf.len() {
    cur.row += 1;
  }
  truncate_cursor_to_line(cur, buf, size);
  align_cursor(cur, size);
}

fn return_cursor(cur: &mut Cursor) {
  cur.row += 1;
  cur.col = 0;
}

// Editing helpers
fn break_line_and_return_cursor(cur: &mut Cursor, buf: &mut Buffer) {
  break_line_at(cur, buf);
  return_cursor(cur);
}

fn insert_and_move_cursor(
  ch: char,
  cur: &mut Cursor,
  buf: &mut Buffer,
  size: &Size,
) {
  insert_at(ch, cur, buf);
  move_cursor_right(cur, buf, size);
}

fn delete_and_move_cursor(
  cur: &mut Cursor,
  buf: &mut Buffer,
  size: &Size,
) {
  if cur.col == 0 {
    if cur.row > 0 {
      let new_col = buf[cur.row - 1].len();
      merge_into_above(cur, buf);
      cur.row -= 1;
      cur.col = new_col;
    }
  } else {
    delete_at(cur, buf);
    move_cursor_left(cur, buf, size);
  }
}

fn edit_buffer(path: &str, buf: &mut Buffer) -> io::Result<()> {
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
      Key::Char('\n') => break_line_and_return_cursor(&mut cur, buf),
      Key::Char(ch) => insert_and_move_cursor(ch, &mut cur, buf, &size),
      Key::Backspace => delete_and_move_cursor(&mut cur, buf, &size),
      Key::Ctrl('s') => write_file(path, buf)?,
      _ => break,
    }
    update_screen(&mut scr, &cur, buf, &size)?;
  }
  clear_screen(&mut scr)
}

fn main() -> io::Result<()> {
  match env::args().skip(1).next() {
    Some(path) => {
      let mut buf = read_file(&path)?;
      init_buffer_if_empty(&mut buf);
      edit_buffer(&path, &mut buf)
    }
    None => Ok(()),
  }
}
