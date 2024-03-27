extern crate crossterm;
extern crate unicode_width;

use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::{cursor, queue, terminal};
use crossterm::{
    event, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{Stdout, Write};
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use unicode_width::UnicodeWidthStr;

use crate::core::checker::get_default_config_path;
use crate::core::selector::{RendererOperation, SelectOptions, SelectStatus};
use crate::render::const_content::get_bye;

use super::const_content::{get_banner, get_bottom_tips, get_help};

pub struct StyledContent {
    pub content: String,
    pub front_color: Color,
    pub back_color: Color,
}
impl StyledContent {
    pub fn empty() -> Self {
        return StyledContent { content: "".to_owned(), front_color: Color::Reset, back_color: Color::Reset };
    }
    pub fn print(&self, stdout: &mut Stdout) {
        queue!(stdout, SetForegroundColor(self.front_color), SetBackgroundColor(self.back_color), Print(&self.content), ResetColor).unwrap();
    }
}
struct FixedContent {
    pub point: String,
    pub space: String,
    pub prefix: String,
    pub path: String,
    pub end: String,
    pub tail: String,
}

fn to_target_length(s: String, target_length: usize) -> (String, usize) {
    let real_length = UnicodeWidthStr::width(&s[0..]);
    if target_length == real_length || target_length == real_length - 1 || target_length > real_length {
        return (s, real_length);
    } else {
        let half = (real_length + target_length) / 2;
        let res: String = s.char_indices().take(half).map(|(_, i)| i).collect();
        return to_target_length(res, target_length);
    }
}

fn file_fix_length(max_col: u16, path: &str, prefix: &str, end: &str) -> FixedContent {
    if UnicodeWidthStr::width(prefix) + UnicodeWidthStr::width(path) + UnicodeWidthStr::width(end) > max_col as usize {
        let target_length = (max_col as i64 - (max_col as i64 / 4) * 3) - end.len() as i64;
        let space_length = 2;
        //it may overflow?
        let (head, head_length) = to_target_length(prefix.to_owned() + path, target_length as usize);
        let reversed: String = path.char_indices().rev().map(|(_, i)| i).collect();
        let (tail_reversed, tail_length) = to_target_length(reversed, max_col as usize - head_length - space_length - end.len() - 4);
        let tail: String = tail_reversed.char_indices().rev().map(|(_, i)| i).collect();
        // let tail_width = UnicodeWidthStr::width(&tail[0..]);

        return FixedContent { space: " ".repeat(space_length), point: ".".repeat(max_col as usize - head_length - tail_length - end.len() - space_length), path: head.replace(prefix, ""), prefix: prefix.to_owned(), end: end.to_owned(), tail };
    }
    return FixedContent { space: " ".repeat(max_col as usize - UnicodeWidthStr::width(prefix) - UnicodeWidthStr::width(path) - UnicodeWidthStr::width(end)), point: "".to_owned(), tail: "".to_owned(), path: path.to_owned(), prefix: prefix.to_owned(), end: end.to_owned() };
}

fn print_content(stdout: &mut Stdout, option: &SelectOptions, fixed_content: FixedContent, selected: bool) {
    let content = fixed_content.path + &fixed_content.point + &fixed_content.tail + &fixed_content.space + &fixed_content.end + "\r\n"; //#TODO can make it more flexible
    let prefix_printer = StyledContent { content: fixed_content.prefix.clone(), front_color: Color::Yellow, back_color: Color::Reset };
    let mut content_printer = StyledContent { content: content.clone(), front_color: Color::Reset, back_color: Color::Reset };
    match option.status {
        SelectStatus::Live | SelectStatus::Calculating | SelectStatus::Searched => {
            prefix_printer.print(stdout);
            if selected == true {
                content_printer.back_color = Color::Blue;
            }
            content_printer.print(stdout);
        }
        SelectStatus::Deleted => {
            prefix_printer.print(stdout);
            content_printer.front_color = Color::Red;
            if selected == true {
                content_printer.back_color = Color::Grey;
            }
            content_printer.print(stdout);
        }
        SelectStatus::Deleting => {
            prefix_printer.print(stdout);
            content_printer.front_color = Color::DarkYellow;
            if selected == true {
                content_printer.back_color = Color::Green;
            }
            content_printer.print(stdout);
        }
        SelectStatus::System => {
            content_printer.front_color = Color::Cyan;
            if selected == true {
                content_printer.back_color = Color::Blue;
            }
            content_printer.print(stdout);
        }
    }
}

pub fn refresh_selector(stdout: &mut Stdout, _options: Arc<Mutex<Vec<SelectOptions>>>, selected: &mut usize, top_content: &StyledContent, bottom_content: &StyledContent) {
    queue!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine), cursor::MoveTo(0, 0), cursor::Hide).unwrap();
    top_content.print(stdout);
    let mut row = 0;
    let (_max_col, _max_row) = crossterm::terminal::size().unwrap();
    let max_row = _max_row as usize - top_content.content.match_indices("\r\n").count(); //remain rows after counting the top_content
    let options = _options.lock().unwrap();
    for i in 0..options.len() {
        let fixed_content = file_fix_length(_max_col, &options[i].path, &options[i].prefix, &options[i].end);
        if *selected == i {
            print_content(stdout, &options[i], fixed_content, true);
        } else {
            if options.len() > max_row - 1 {
                // if i>*selected+((max_row as usize)-row-2){
                //     return;
                // }
                if (*selected) as i64 - ((max_row / 2) as i64) < i as i64 && (*selected) as i64 > i as i64 {
                    print_content(stdout, &options[i], fixed_content, false);
                    row += 1;
                } else if *selected + ((max_row as usize) - row - 1) > i && *selected < i {
                    print_content(stdout, &options[i], fixed_content, false);
                } else if *selected + ((max_row as usize) - row - 1) <= i {
                    break;
                }
            } else {
                print_content(stdout, &options[i], fixed_content, false);
            }
        }
    }

    queue!(stdout, terminal::Clear(terminal::ClearType::FromCursorDown),).unwrap();

    if bottom_content.content != "" {
        bottom_content.print(stdout);
    }
    stdout.flush().unwrap();
}

fn select_down(stdout: &mut Stdout, _options: Arc<Mutex<Vec<SelectOptions>>>, selected: &mut usize, size: usize, title: &StyledContent, bottom: &StyledContent) {
    {
        let options = _options.lock().unwrap();
        if options.len() == 0 {
            return;
        }
        if *selected as i64 >= options.len() as i64 - size as i64 {
            *selected = 0;
        } else {
            *selected += size;
        }
    }
    refresh_selector(stdout, _options, selected, title, bottom)
}
fn select_up(stdout: &mut Stdout, _options: Arc<Mutex<Vec<SelectOptions>>>, selected: &mut usize, size: usize, title: &StyledContent, bottom: &StyledContent) {
    {
        let options = _options.lock().unwrap();
        if options.len() == 0 {
            return;
        }
        if *selected as i64 - size as i64 <= -1 {
            *selected = options.len() - 1;
        } else {
            *selected -= size;
        }
    }
    refresh_selector(stdout, _options, selected, title, bottom)
}
pub fn selector(stdout: &mut Stdout, options: Arc<Mutex<Vec<SelectOptions>>>, selected: &mut usize, need_refresh: Arc<AtomicBool>) -> (usize, RendererOperation) {
    let top_content = StyledContent { content: get_banner(), front_color: Color::Magenta, back_color: Color::Reset };
    let input = String::new();
    execute!(stdout, terminal::EnterAlternateScreen).unwrap();
    terminal::enable_raw_mode().unwrap();
    let input_bottom_content = StyledContent { content: get_bottom_tips(), front_color: Color::Blue, back_color: Color::Reset };
    refresh_selector(stdout, options.clone(), selected, &top_content, &input_bottom_content);
    if let Some(value) = handle_key_event(input, input_bottom_content, stdout, options, selected, top_content, need_refresh) {
        return value;
    }

    (selected.to_owned(), RendererOperation::NONE)
}

fn handle_key_event(mut input: String, mut input_bottom_content: StyledContent, stdout: &mut Stdout, options: Arc<Mutex<Vec<SelectOptions>>>, selected: &mut usize, top_content: StyledContent, need_refresh: Arc<AtomicBool>) -> Option<(usize, RendererOperation)> {
    let mut update = true;
    loop {
        if event::poll(Duration::from_millis(400)).unwrap() {
            if input.starts_with("-") == false {
                //When there is not any in Input
                input_bottom_content.content = get_bottom_tips();
                match event::read().unwrap() {
                    Event::Key(ke) => {
                        if ke.kind == KeyEventKind::Press {
                            match ke.code {
                                KeyCode::Right => {
                                    select_down(stdout, options.clone(), selected, 10, &top_content, &input_bottom_content);
                                }
                                KeyCode::Left => select_up(stdout, options.clone(), selected, 10, &top_content, &input_bottom_content),
                                KeyCode::Char(c) => {
                                    if c == 'q' {
                                        return Some((usize::MAX, RendererOperation::SYSTEM));
                                    } else if c == 'j' {
                                        select_down(stdout, options.clone(), selected, 1, &top_content, &input_bottom_content);
                                    } else if c == 'k' {
                                        select_up(stdout, options.clone(), selected, 1, &top_content, &input_bottom_content);
                                    } else if c == 'b' {
                                        select_up(stdout, options.clone(), selected, 10, &top_content, &input_bottom_content)
                                    } else if c == 'f' {
                                        select_down(stdout, options.clone(), selected, 10, &top_content, &input_bottom_content);
                                    } else if c == '-' {
                                        input += "-";
                                        input_bottom_content.content = input.clone();
                                        refresh_selector(stdout, options.clone(), selected, &top_content, &input_bottom_content);
                                    } else if c == ' ' {
                                        return Some((selected.to_owned(), RendererOperation::REMOVE));
                                    }
                                }
                                KeyCode::Enter => {
                                    return Some((selected.to_owned(), RendererOperation::REMOVE));
                                }
                                KeyCode::Down => {
                                    select_down(stdout, options.clone(), selected, 1, &top_content, &input_bottom_content);
                                }
                                KeyCode::Up => {
                                    select_up(stdout, options.clone(), selected, 1, &top_content, &input_bottom_content);
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            } else {
                if update == true {
                    match event::read().unwrap() {
                        Event::Key(ke) => {
                            if ke.kind == KeyEventKind::Press {
                                match ke.code {
                                    KeyCode::Char(c) => {
                                        input += &c.to_string();
                                        input_bottom_content.content = input.clone();
                                        refresh_selector(stdout, options.clone(), selected, &top_content, &input_bottom_content);
                                    }
                                    KeyCode::Backspace => {
                                        if input.len() > 0 {
                                            input = input[0..input.len() - 1].to_owned();
                                        }
                                        input_bottom_content.content = input.clone();
                                        refresh_selector(stdout, options.clone(), selected, &top_content, &input_bottom_content);
                                    }
                                    KeyCode::Enter => {
                                        if input.contains("-config") {
                                            if let Some(config_path) = get_default_config_path() {
                                                open_parent_path(&config_path, stdout, &top_content, &mut update);
                                            }
                                        } else if input.contains("-open") {
                                            let guard = options.lock().unwrap();
                                            open_parent_path(&guard[*selected].path, stdout, &top_content, &mut update);
                                        } else if input.contains("-help") {
                                            print_help(stdout, &top_content);
                                            update = false
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                } else {
                    match event::read().unwrap() {
                        Event::Key(ke) => {
                            if ke.kind == KeyEventKind::Press {
                                match ke.code {
                                    KeyCode::Char(c) => {
                                        if c == 'q' {
                                            update = true
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            if update == true {
                refresh_selector(stdout, options.clone(), selected, &top_content, &input_bottom_content)
            }
        } else {
            if need_refresh.load(std::sync::atomic::Ordering::Relaxed) == true && update == true {
                refresh_selector(stdout, options.clone(), selected, &top_content, &input_bottom_content);
                need_refresh.swap(false, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }
}

pub fn exit(stdout: &mut Stdout) {
    execute!(stdout, ResetColor, cursor::Show, terminal::LeaveAlternateScreen).unwrap();
    terminal::disable_raw_mode().unwrap();
    queue!(stdout, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0), SetForegroundColor(Color::Magenta), Print(get_banner()), ResetColor, SetForegroundColor(Color::Green), Print(get_bye()), ResetColor).unwrap();
    stdout.flush().unwrap();
}

fn print_help(stdout: &mut Stdout, top_content: &StyledContent) {
    let content = &get_help();
    print_in_another_screen(stdout, top_content, content);
}

fn print_error(stdout: &mut Stdout, top_content: &StyledContent, error: &str) {
    print_in_another_screen(stdout, top_content, &format!("Error:\n{}", error));
}

fn print_in_another_screen(stdout: &mut Stdout, top_content: &StyledContent, content: &str) {
    queue!(stdout, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0)).unwrap();
    top_content.print(stdout);
    for line in content.split('\n') {
        queue!(stdout, Print(line), cursor::MoveToNextLine(1)).unwrap();
    }
    queue!(stdout, Print("Click [q] to back\n"), cursor::MoveToNextLine(1)).unwrap();
    stdout.flush().unwrap();
}

fn open_parent_path(p: &str,stdout: &mut Stdout, top_content: &StyledContent,update:&mut bool) {
    let path = Path::new(p);
    if let Some(p) = path.parent() {
        if let Err(_) = open::that(p) {
            print_error(stdout, &top_content, &format!("Can not open: {}\n", p.to_str().unwrap()));
            *update = false
        };
    }
}
