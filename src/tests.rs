use super::*;

use tempfile;

#[test]
fn test_size() {
  let size = get_screen_size().unwrap();
  assert!(size.cols > size.rows);
}

#[test]
fn test_file_system() {
  let dir = tempfile::tempdir().unwrap();

  { // read missing file
    let path = dir.path().join("missing");
    let result = read_file(&path.to_str().unwrap());
    assert!(result.is_ok());
    let buffer = result.unwrap();
    assert_eq!(0, buffer.len());
  }

  { // write buffer to file
    let path = dir.path().join("new");
    let mut buffer = Buffer::new();
    buffer.push(Line::from("test"));
    let result = write_file(&path.to_str().unwrap(), &buffer);
    assert!(result.is_ok());
  }

  { // read existing file
    let path = dir.path().join("new");
    let result = read_file(&path.to_str().unwrap());
    assert!(result.is_ok());
    let buffer = result.unwrap();
    assert_eq!(1, buffer.len());
    assert_eq!(Line::from("test"), buffer[0]);
  }
}

fn check_range(
  cur: &Cursor,
  size: &Size,
  l: Range<usize>,
  c: Range<usize>,
) {
  assert_eq!(l, buffer_line_range(cur, size));
  assert_eq!(c, buffer_char_range(cur, size));
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

  let mut apply_and_check =
    |f: fn(&mut Cursor, &Buffer, &Size), l: Range<usize>, c: Range<usize>| {
      f(&mut cur, &buf, &size);
      check_range(&cur, &size, l, c);
  };

  // Trying to move left past the first column should have no effect
  apply_and_check(move_cursor_left, 0..3, 0..2);
  // Trying to move up past the first row should have no effect
  apply_and_check(move_cursor_up, 0..3, 0..2);
  // Moving right without reaching the edge screen should not change the range.
  apply_and_check(move_cursor_right, 0..3, 0..2);
  // Reaching the edge should cause the edge to move
  apply_and_check(move_cursor_right, 0..3, 1..3);
  apply_and_check(move_cursor_right, 0..3, 2..4);
  // Should be able to go one past the size of the buffer
  apply_and_check(move_cursor_right, 0..3, 3..5);
  // Reaching the edge of the buffer should cause the cursor to wrap around
  apply_and_check(move_cursor_right, 0..3, 0..2);
  apply_and_check(move_cursor_left, 0..3, 3..5);
  // Moving down without reaching the edge should not change the range.
  apply_and_check(move_cursor_down, 0..3, 3..5);
  apply_and_check(move_cursor_down, 0..3, 3..5);
  // Reaching the edge should cause the edge to move
  apply_and_check(move_cursor_down, 1..4, 3..5);
  // Reaching the last line should cause the cursor to start a new line
  apply_and_check(move_cursor_down, 2..5, 3..5);
  apply_and_check(move_cursor_down, 3..6, 0..2);
  // Trying to move past the fake last row should have no effect
  apply_and_check(move_cursor_down, 3..6, 0..2);
  // Reaching the edge of the buffer should cause the cursor to wrap around
  apply_and_check(move_cursor_left, 3..6, 3..5);
  // Moving back through the line
  apply_and_check(move_cursor_left, 3..6, 3..5);
  apply_and_check(move_cursor_left, 3..6, 2..4);
  apply_and_check(move_cursor_left, 3..6, 1..3);
  apply_and_check(move_cursor_left, 3..6, 0..2);
  // Moving back through the rows
  apply_and_check(move_cursor_up, 3..6, 0..2);
  apply_and_check(move_cursor_up, 2..5, 0..2);
  apply_and_check(move_cursor_up, 1..4, 0..2);
  apply_and_check(move_cursor_up, 0..3, 0..2);
  apply_and_check(move_cursor_up, 0..3, 0..2);
}
