use super::node_module;
use serde_json::Result;
use std::io::{stdin, stdout, Write};
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::{color, style};

///
/// Function constructs TUI
///
fn build(
  stdout: &mut AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>,
  row: &usize,
  dependency_list: &[&node_module::NodeModule],
) -> Result<()> {
  for (i, node_module) in dependency_list.into_iter().enumerate() {
    let line_number = i + 1_usize;
    let is_current_line = (*row as u32) == (line_number as u32);

    write!(
      stdout,
      "{}",
      termion::cursor::Goto(1, (i as u16) * 2_u16 + 1_u16)
    )
    .expect("set the cursor position");

    if node_module.selected.get() {
      write!(stdout, "{}◉{}", color::Fg(color::Green), style::Reset)
        .expect("write the checkbox icon");
    } else {
      write!(stdout, "◯").expect("write the checkbox icon");
    }

    write!(stdout, " ").expect("just write a space for layout");

    write!(stdout, "{}", node_module.r#type.short).expect("write the type of the node_module");

    write!(stdout, " ").expect("just write a space for layout");

    if is_current_line {
      write!(stdout, "{}", termion::style::Underline).expect("emphasize the current line");
    }

    write!(stdout, "{}@{}", node_module.name, node_module.version)
      .expect("write a the name and version of a node module");

    if is_current_line {
      write!(stdout, "{}", termion::style::NoUnderline).expect("reset emphasized");
    }

    write!(stdout, "\r\n  ").expect("new line and indent");

    write!(
      stdout,
      "{}{}{}",
      color::Fg(color::LightBlack),
      node_module.path.to_str().unwrap(),
      termion::style::Reset
    )
    .expect("write path in gray");
  }

  let size = termion::terminal_size().expect("get the terminal size");
  write!(
    stdout,
    "{}{}",
    termion::cursor::Goto(size.0, size.1),
    termion::cursor::Hide
  )
  .expect("hide the cursor");
  stdout.flush().unwrap();
  Ok(())
}

///
/// Function renders TUI
///
pub fn render(dependency_list: &[&node_module::NodeModule]) {
  let stdin = stdin();
  let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());
  let mut row: usize = 1;
  write!(stdout, "{}", termion::clear::All).expect("clear terminal");
  build(&mut stdout, &row, &dependency_list).expect("show dependencies checkboxes");
  stdout.flush().unwrap();

  for c in stdin.events() {
    let event = c.unwrap();

    match event {
      Event::Key(Key::Char('q')) | Event::Key(Key::Ctrl('c')) => break,
      Event::Key(Key::Char('k')) => {
        if row > 1 {
          row -= 1;
        }
      }
      Event::Key(Key::Char('j')) => {
        if row < dependency_list.len() {
          row += 1;
        }
      }
      // space
      Event::Key(Key::Char(' ')) => {
        dependency_list
          .get(row - 1_usize)
          .map(|&node_module| node_module.selected.set(!node_module.selected.get()));
      }
      // enter
      Event::Key(Key::Char('\n')) => {
        break;
      }
      Event::Key(_) | Event::Unsupported(_) | Event::Mouse(_) => {}
    }

    build(&mut stdout, &row, &dependency_list).expect("rendering");
  }
}
