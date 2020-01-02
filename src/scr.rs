use std::io::{self, BufWriter, Write};
use std::ops::Add;

use termion::{self, raw::IntoRawMode};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
  pub row: u16,
  pub col: u16,
}

impl Add for Position {
  type Output = Self;

  fn add(self, r: Self) -> Self {
    Self{row: self.row + r.row, col: self.col + r.col}
  }
}

#[derive(Clone, Copy, Debug)]
pub struct Size {
  pub rows: u16,
  pub cols: u16,
}

impl Size {
  fn contains(&self, pos: &Position) -> bool {
    pos.row < self.rows && pos.col < self.cols
  }
}

pub trait Screen {
  fn put_at(&mut self, s: &str, pos: Position) -> io::Result<()>;
  fn flush(&mut self) -> io::Result<()>;
  fn size(&self) -> io::Result<Size>;
}

struct TermionScreen<T: Write> {
  writer: T,
}

pub fn init_screen() -> io::Result<impl Screen> {
  termion::screen::AlternateScreen::from(io::stdout())
    .into_raw_mode()
    .map(BufWriter::new)
    .map(TermionScreen::new)
}

impl<T: Write> TermionScreen<T> {
  pub fn new(writer: T) -> Self {
    TermionScreen{writer}
  }

  pub fn into_inner(self) -> T {
    self.writer
  }
}

impl<T: Write> Screen for TermionScreen<T> {
  fn put_at(&mut self, s: &str, pos: Position) -> io::Result<()> {
    if !self.size()?.contains(&pos) {
      panic!("tried to put at position outside screen");
    }
    write!(
      self.writer,
      "{}{}",
      termion::cursor::Goto(pos.col + 1 as u16, pos.row + 1 as u16),
      s,
    )
  }
  fn flush(&mut self) -> io::Result<()> {
    self.writer.flush()
  }
  fn size(&self) -> io::Result<Size> {
    termion::terminal_size().map(|(cols, rows)| Size{rows, cols})
  }
}

pub struct Window {
  pub position: Position,
  pub size: Size,
}

impl Window {
  pub fn with_size(size: Size) -> Self {
    Window{position: Position{row: 0, col: 0}, size}
  }
  pub fn with_size_at(size: Size, position: Position) -> Self {
    Window{position, size}
  }
  pub fn put_at(
    &self,
    s: &str,
    pos: Position,
    scr: &mut dyn Screen,
  ) -> io::Result<()> {
    println!("{:?}", self.position);
    scr.put_at(s, self.position + pos)
  }
  pub fn blank(&self, scr: &mut dyn Screen) -> io::Result<()> {
    for row in 0..self.size.rows {
      for col in 0..self.size.cols {
        self.put_at(" ", Position{row, col}, scr)?;
      }
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use crate::tests::assert_panics;

  pub enum TestScreenCall {
    PutAt(Box<dyn Fn(&str, Position) -> io::Result<()>>),
  }

  impl TestScreenCall {
    fn put_at<F: Fn(&str, Position) -> io::Result<()> + 'static>(call: F) -> Self {
      TestScreenCall::PutAt(Box::new(call))
    }
  }

  pub struct TestScreen {
    calls: Vec<TestScreenCall>,
    call: usize,
  }

  impl TestScreen {
    fn new(calls: Vec<TestScreenCall>) -> Self {
      TestScreen{calls, call: 0}
    }

    fn assert_call_count(&self) {
      assert_eq!(self.call, self.calls.len());
    }
  }

  impl Screen for TestScreen {
    fn put_at(&mut self, s: &str, pos: Position) -> io::Result<()> {
      let res = match &self.calls[self.call] {
        TestScreenCall::PutAt(f) => f(s, pos),
      };
      self.call += 1;
      res
    }
    fn flush(&mut self) -> io::Result<()> {
      unimplemented!();
    }
    fn size(&self) -> io::Result<Size> {
      unimplemented!();
    }
  }

  pub fn check_put_at<S: Into<String>>(es: S, epos: Position) -> TestScreenCall {
    let ess = es.into();
    TestScreenCall::put_at(move |s, pos| {
      assert_eq!(ess.as_str(), s);
      assert_eq!(epos, pos);
      Ok(())
    })
  }

  #[test]
  fn test_termion_screen() {
    let mut scr = TermionScreen::new(Vec::new());
    let size = scr.size().unwrap();
    assert_eq!(24, size.rows);
    assert_eq!(80, size.cols);
    scr.put_at("abc", Position{row: 0, col: 0}).unwrap();
    scr.put_at("def", Position{row: 10, col: 5}).unwrap();
    scr.put_at("ghi", Position{row: 23, col: 79}).unwrap();
    scr.flush().unwrap(); 
    let buf = scr.into_inner();
    let exp = format!(
      "{}abc{}def{}ghi",
      termion::cursor::Goto(1, 1), 
      termion::cursor::Goto(6, 11), 
      termion::cursor::Goto(80, 24), 
    );
    assert_eq!(exp.as_bytes(), buf.as_slice());
    assert_panics(|| {
      let mut scr = TermionScreen::new(Vec::new());
      scr.put_at("jkl", Position{row: 24, col: 0}).unwrap();
    });
    assert_panics(|| {
      let mut scr = TermionScreen::new(Vec::new());
      scr.put_at("jkl", Position{row: 0, col: 80}).unwrap();
    });
  }

  #[test]
  fn test_put_at_window() {
    {
      let win = Window::with_size(Size{rows: 10, cols: 10});
      let mut mock = TestScreen::new(vec![
        check_put_at("abc", Position{row: 0, col: 0}),
        check_put_at("def", Position{row: 2, col: 5}),
      ]);
      win.put_at("abc", Position{row: 0, col: 0}, &mut mock).unwrap();
      win.put_at("def", Position{row: 2, col: 5}, &mut mock).unwrap();
      mock.assert_call_count();
    } {
      let win = Window::with_size_at(Size{rows: 10, cols: 10}, Position{row: 2, col: 4});
      let mut mock = TestScreen::new(vec![
        check_put_at("abc", Position{row: 2, col: 4}),
        check_put_at("abc", Position{row: 4, col: 7}),
      ]);
      win.put_at("abc", Position{row: 0, col: 0}, &mut mock).unwrap();
      win.put_at("abc", Position{row: 2, col: 3}, &mut mock).unwrap();
      mock.assert_call_count();
    }
  }

  #[test]
  fn test_blank_window() {
    {
      let win = Window::with_size(Size{rows: 2, cols: 3});
      let mut mock = TestScreen::new(vec![
        check_put_at(" ", Position{row: 0, col: 0}),
        check_put_at(" ", Position{row: 0, col: 1}),
        check_put_at(" ", Position{row: 0, col: 2}),
        check_put_at(" ", Position{row: 1, col: 0}),
        check_put_at(" ", Position{row: 1, col: 1}),
        check_put_at(" ", Position{row: 1, col: 2}),
      ]);
      win.blank(&mut mock).unwrap();
      mock.assert_call_count();
    } {
      let win = Window::with_size_at(Size{rows: 3, cols: 2}, Position{row: 2, col: 4});
      let mut mock = TestScreen::new(vec![
        check_put_at(" ", Position{row: 2, col: 4}),
        check_put_at(" ", Position{row: 2, col: 5}),
        check_put_at(" ", Position{row: 3, col: 4}),
        check_put_at(" ", Position{row: 3, col: 5}),
        check_put_at(" ", Position{row: 4, col: 4}),
        check_put_at(" ", Position{row: 4, col: 5}),
      ]);
      win.blank(&mut mock).unwrap();
      mock.assert_call_count();
    }
  }
}