#[cfg(test)]
extern crate tempfile;
extern crate termion;

#[cfg(test)]
mod tests;

use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::ops::Range;

use termion::{
  raw::IntoRawMode,
  input::TermRead,
};

type Line = String;
type Buffer = Vec<Line>;
type Screen =
   io::BufWriter<termion::raw::RawTerminal<termion::screen::AlternateScreen<io::Stdout>>>;
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
fn read_file(path: &str) -> io::Result<Buffer> {
  match fs::OpenOptions::new().read(true).open(path) {
    Ok(file) => BufReader::new(file).lines().collect(),
    Err(err) => match err.kind() {
      io::ErrorKind::NotFound => Ok(Buffer::new()),
      _ => Err(err),
    }
  }
}

fn write_file(path: &str, buf: &Buffer) -> io::Result<()> {
  let mut file = fs::OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .truncate(true)
    .open(path)?;
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
  if cur.row > buf.len() {
    panic!("tried to insert past end of buffer");
  }
  buf[cur.row].insert(cur.col, ch)
}

fn delete_before(cur: &Cursor, buf: &mut Buffer) {
  if cur.col == 0 {
    panic!("tried to delete before start of buffer");
  }
  buf[cur.row].remove(cur.col - 1);
}

fn delete_at(cur: &Cursor, buf: &mut Buffer) {
  if cur.row >= buf.len() {
    panic!("tried to delete after end of buffer");
  }
  if cur.col >= buf[cur.row].len() {
    panic!("tried to delete after end of line");
  }
  buf[cur.row].remove(cur.col);
}

fn merge_next_line_into(cur: &Cursor, buf: &mut Buffer) {
  if cur.row + 1 >= buf.len() {
    panic!("tried to merge line from past end of buffer");
  }
  let line = buf.remove(cur.row + 1);
  buf[cur.row].push_str(&line);
}

fn break_line_at(cur: &Cursor, buf: &mut Buffer) {
  let new_line = buf[cur.row].split_off(cur.col);
  buf.insert(cur.row + 1, new_line);
}

fn push_new_line_if_at_end(cur: &Cursor, buf: &mut Buffer) {
  if cur.row == buf.len() {
    buf.push(Line::new());
  }
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

fn blank_screen(scr: &mut Screen) -> io::Result<()> {
  write!(scr, "{}{}", termion::cursor::Goto(1, 1), termion::clear::All)
}

fn init_screen() -> io::Result<Screen> {
  termion::screen::AlternateScreen::from(io::stdout())
    .into_raw_mode().map(BufWriter::new)
}

fn update_screen(
  scr: &mut Screen,
  cur: &Cursor,
  buf: &Buffer,
  size: &Size,
) -> io::Result<()> {
  blank_screen(scr)?;
  write_buffer_to_screen(scr, cur, buf, size)?;
  scr.flush()
}

// Cursor movement
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

fn move_cursor_up(cur: &mut Cursor, buf: &Buffer, size: &Size) {
  if cur.row > 0 {
    cur.row -= 1;
  } else {
    cur.row = buf.len();
  }
  truncate_cursor_to_line(cur, buf);
  align_cursor(cur, size);
}

fn move_cursor_down(cur: &mut Cursor, buf: &Buffer, size: &Size) {
  if cur.row < buf.len() {
    cur.row += 1;
  } else {
    cur.row = 0;
  }
  truncate_cursor_to_line(cur, buf);
  align_cursor(cur, size);
}

fn move_cursor_end_of_prev_line(cur: &mut Cursor, buf: &Buffer, size: &Size) {
  if cur.row == 0 {
    panic!("tried to move cursor before start of buffer");
  }
  cur.row -= 1;
  cur.col = buf[cur.row].len();
  align_cursor(cur, size);
}

fn move_cursor_start_of_next_line(cur: &mut Cursor, buf: &Buffer, size: &Size) {
  if cur.row >= buf.len() {
    panic!("tried to move cursor past end of buffer");
  }
  cur.row += 1;
  cur.col = 0;
  align_cursor(cur, size);
}

fn is_whitespace(c: char) -> bool {
  match c {
    ' ' => true,
    '\n' => true,
    '\t' => true,
    _ => false,
  }
}

fn is_blank(cur: &mut Cursor, buf: &Buffer) -> bool {
  cur.row >= buf.len() || buf[cur.row].len() == cur.col || is_whitespace(buf[cur.row].as_bytes()[cur.col] as char)
}

fn is_blank_line(cur: &mut Cursor, buf: &Buffer) -> bool {
  if cur.row < buf.len() && buf[cur.row].len() > 0 {
    for c in buf[cur.row].chars() {
      if !is_whitespace(c) {
        return false;
      }
    }
  }
  true
}

fn move_cursor_to_next_blank(cur: &mut Cursor, buf: &Buffer, size: &Size) {
  move_cursor_right(cur, buf, size);
  while !is_blank(cur, buf) {
    move_cursor_right(cur, buf, size);
  }
}

fn move_cursor_to_prev_blank(cur: &mut Cursor, buf: &Buffer, size: &Size) {
  move_cursor_left(cur, buf, size);
  while !is_blank(cur, buf) {
    move_cursor_left(cur, buf, size);
  }
}

fn move_cursor_to_next_blank_line(cur: &mut Cursor, buf: &Buffer, size: &Size) {
  move_cursor_down(cur, buf, size);
  while !is_blank_line(cur, buf) {
    move_cursor_down(cur, buf, size);
  }
}

fn move_cursor_to_prev_blank_line(cur: &mut Cursor, buf: &Buffer, size: &Size) {
  move_cursor_up(cur, buf, size);
  while !is_blank_line(cur, buf) {
    move_cursor_up(cur, buf, size);
  }
}

fn align_cursor(cur: &mut Cursor, size: &Size) {
  if cur.col < cur.left {
    cur.left = cur.col;
  }
  if cur.col > cur.left + size.cols - 1{
    cur.left = cur.col - size.cols + 1;
  }
  if cur.row < cur.top {
    cur.top = cur.row;
  }
  if cur.row > cur.top + size.rows - 1{
    cur.top = cur.row - size.rows + 1;
  }
}

fn truncate_cursor_to_line(cur: &mut Cursor, buf: &Buffer) {
  if cur.row < buf.len() {
    if cur.col > buf[cur.row].len() {
      cur.col = buf[cur.row].len();
    }
  } else {
    cur.col = 0;
  }
}

// Editing helpers
fn break_line_and_return_cursor(cur: &mut Cursor, buf: &mut Buffer, size: &Size) {
  break_line_at(cur, buf);
  move_cursor_start_of_next_line(cur, buf, size);
}

fn insert_and_move_cursor(
  ch: char,
  cur: &mut Cursor,
  buf: &mut Buffer,
  size: &Size,
) {
  push_new_line_if_at_end(cur, buf);
  insert_at(ch, cur, buf);
  move_cursor_right(cur, buf, size);
}

fn delete_in_place(cur: &mut Cursor, buf: &mut Buffer, _size: &Size) {
  if cur.row < buf.len() && cur.col < buf[cur.row].len() {
    delete_at(cur, buf);
  } else if cur.row + 1 < buf.len() && cur.col == buf[cur.row].len() {
    merge_next_line_into(cur, buf);
  }
}

fn delete_and_move_cursor(cur: &mut Cursor, buf: &mut Buffer, size: &Size) {
  if cur.col > 0 {
    delete_before(cur, buf);
    move_cursor_left(cur, buf, size);
  } else if cur.row > 0 {
    move_cursor_end_of_prev_line(cur, buf, size);
    if cur.row + 1 < buf.len() {
      merge_next_line_into(cur, buf);
    }
  }
}

fn delete_line(cur: &mut Cursor, src: &mut Buffer, size: &Size) {
  src.remove(cur.row);
  truncate_cursor_to_line(cur, src);
  align_cursor(cur, size);
}

fn cut_line(cur: &mut Cursor, src: &mut Buffer, dst: &mut Buffer, size: &Size) {
  dst.push(src.remove(cur.row));
  truncate_cursor_to_line(cur, src);
  align_cursor(cur, size);
}

fn copy_line(cur: &mut Cursor, src: &Buffer, dst: &mut Buffer, size: &Size) {
  src.get(cur.row).map(|line| dst.push(line.clone()));
  move_cursor_down(cur, src, size);
}

fn paste_line(cur: &mut Cursor, src: &mut Buffer, dst: &mut Buffer, size: &Size) {
  src.pop().map(|line| dst.insert(cur.row, line));
  truncate_cursor_to_line(cur, dst);
  align_cursor(cur, size);
}

enum Mode {
  Insert,
  Normal,
  Quit,
}

fn handle_key_insert_mode(
  key: Key,
  cur: &mut Cursor,
  buf: &mut Buffer,
  size: &Size
) -> io::Result<Mode> {
  match key {
    Key::Char('\n') => break_line_and_return_cursor(cur, buf, size),
    Key::Char(ch) => insert_and_move_cursor(ch, cur, buf, size),
    Key::Delete => delete_in_place(cur, buf, size),
    Key::Backspace => delete_and_move_cursor(cur, buf, size),
    Key::Esc => return Ok(Mode::Normal),
    _ => (),
  };
  Ok(Mode::Insert)
}

fn handle_key_normal_mode(
  key: Key,
  path: &str,
  cur: &mut Cursor,
  buf: &mut Buffer,
  clip: &mut Buffer,
  size: &Size
) -> io::Result<Mode> {
  match key {
    Key::Char('i') => return Ok(Mode::Insert),
    // movement
    Key::Char('h') => move_cursor_left(cur, buf, size),
    Key::Char('l') => move_cursor_right(cur, buf, size),
    Key::Char('k') => move_cursor_up(cur, buf, size),
    Key::Char('j') => move_cursor_down(cur, buf, size),
    Key::Char('H') => move_cursor_to_prev_blank(cur, buf, size),
    Key::Char('L') => move_cursor_to_next_blank(cur, buf, size),
    Key::Char('K') => move_cursor_to_prev_blank_line(cur, buf, size),
    Key::Char('J') => move_cursor_to_next_blank_line(cur, buf, size),
    // cut-paste buffer
    Key::Char('d') => delete_line(cur, buf, size),
    Key::Char('c') => copy_line(cur, buf, clip, size),
    Key::Char('v') => paste_line(cur, clip, buf, size),
    Key::Char('x') => cut_line(cur, buf, clip, size),
    Key::Char('s') => write_file(path, buf)?,
    Key::Char('q') => return Ok(Mode::Quit),
    _ => (),
  };
  Ok(Mode::Normal)
}

fn edit_buffer(path: &str, buf: &mut Buffer) -> io::Result<()> {
  let mut scr = init_screen()?;
  let mut cur = Cursor::new();
  let mut clip = Buffer::new();
  let mut size = get_screen_size()?;
  let mut mode = Mode::Normal;
  update_screen(&mut scr, &cur, buf, &size)?;
  for res in io::stdin().keys() {
    let key = res?;
    size = get_screen_size()?;
    mode = match mode {
      Mode::Insert => handle_key_insert_mode(key, &mut cur, buf, &size)?,
      Mode::Normal => handle_key_normal_mode(key, path, &mut cur, buf, &mut clip, &size)?,
      _ => Mode::Quit,
    };
    match mode {
      Mode::Quit => break,
      _ => (),
    }
    update_screen(&mut scr, &cur, buf, &size)?;
  }
  Ok(())
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
