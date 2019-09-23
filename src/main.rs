/*
[x] Create a new file and open it
[x] Open an existing file
[ ] Navigate to a location in an open file
[ ] Insert a character at the current location
[ ] Delete a character at the current location
[ ] Write an open file out
*/

extern crate termion;

use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};

use termion::{
  raw::IntoRawMode,
  input::TermRead,
};

type Buffer = Vec<String>;

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

type Cursor = (usize, usize);

fn insert_at(ch: u8, cur: &mut Cursor, buf: &mut Buffer) {
}

fn delete_at(cur: &mut Cursor, buf: &mut Buffer) {
}

type Screen = termion::raw::RawTerminal<io::Stdout>;

fn write_buffer_to_screen(
  scr: &mut Screen,
  cur: &Cursor,
  buf: &mut Buffer,
) -> io::Result<()> {
  Ok(())
}

fn clear_screen(scr: &mut Screen) -> io::Result<()> {
  Ok(())
}

fn update_screen(
  scr: &mut Screen,
  cur: &Cursor,
  buf: &mut Buffer,
) -> io::Result<()> {
  clear_screen(scr)?;
  write_buffer_to_screen(scr, cur, buf)
}

fn init_screen() -> io::Result<Screen> {
  io::stdout().into_raw_mode()
}

enum Dir { Left, Right, Up, Down }

fn update_cursor(cur: &mut Cursor, buf: &Buffer, dir: Dir) {
  match dir {
    Dir::Left => (),
    Dir::Right => (),
    Dir::Up => (),
    Dir::Down => (),
  }
}
type Key = termion::event::Key;

fn edit_buffer(buf: &mut Buffer) -> io::Result<()> {
  let mut scr = init_screen()?;
  let mut cur = (0, 0);
  for res in io::stdin().keys() {
    let key = res?;
    update_screen(&mut scr, &cur, buf)?;
    match key {
      Key::Left => update_cursor(&mut cur, buf, Dir::Left),
      Key::Right => update_cursor(&mut cur, buf, Dir::Right),
      Key::Up => update_cursor(&mut cur, buf, Dir::Up),
      Key::Down => update_cursor(&mut cur, buf, Dir::Down),
      Key::Char(ch) => insert_at(ch as u8, &mut cur, buf),
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
