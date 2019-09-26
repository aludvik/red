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

fn insert_at(ch: char, cur: &mut Cursor, buf: &mut Buffer) {
  buf[cur.row].insert(cur.col, ch)
}

fn delete_at(cur: &mut Cursor, buf: &mut Buffer) {
  buf[cur.row].remove(cur.col);
}

type Screen = termion::raw::RawTerminal<io::Stdout>;

struct Size {
  cols: usize,
  rows: usize,
}

fn write_line_to_screen(
  scr: &mut Screen,
  cur: &Cursor,
  line: &Line,
  size: &Size,
) -> io::Result<()> {
  let bytes = line.as_bytes();
  for i in cur.left..(cur.left + size.cols) {
    write!(scr, "{}", bytes[i])?;
  }
  Ok(())
}

fn write_buffer_to_screen(
  scr: &mut Screen,
  cur: &Cursor,
  buf: &Buffer,
  size: &Size,
) -> io::Result<()> {
  for i in cur.top..(cur.top + size.rows) {
    write_line_to_screen(scr, cur, &buf[i], size)?;
  }
  write!(scr, "{}", termion::cursor::Goto(
    (cur.row - cur.top + 1) as u16,
    (cur.col - cur.left + 1) as u16,
  ))?;
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
  if cur.col < buf[cur.row].len() - 1 {
    cur.col += 1;
  }
  align_cursor(cur, size);
}

fn truncate_cursor_to_line(cur: &mut Cursor, buf: &Buffer, size: &Size) {
  if cur.col >= buf[cur.row].len() {
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
  if cur.row < buf.len() - 1 {
    cur.row += 1;
  }
  truncate_cursor_to_line(cur, buf, size);
  align_cursor(cur, size);
}

type Key = termion::event::Key;

fn edit_buffer(buf: &mut Buffer) -> io::Result<()> {
  let mut scr = init_screen()?;
  let mut cur = Cursor{col: 0, row: 0, left: 0, top: 0};
  for res in io::stdin().keys() {
    let key = res?;
    let size = termion::terminal_size()
      .map(|(rows, cols)| Size{rows: rows as usize, cols: cols as usize})?;
    update_screen(&mut scr, &cur, buf, &size)?;
    match key {
      Key::Left => move_cursor_left(&mut cur, buf, &size),
      Key::Right => move_cursor_right(&mut cur, buf, &size),
      Key::Up => move_cursor_up(&mut cur, buf, &size),
      Key::Down => move_cursor_down(&mut cur, buf, &size),
      Key::Char(ch) => insert_at(ch, &mut cur, buf),
      Key::Backspace => delete_at(&mut cur, buf),
      _ => break,
    }
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
