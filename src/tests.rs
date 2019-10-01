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
