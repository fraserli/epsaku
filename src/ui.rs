use crate::epub::Epub;

use std::io::Write;

use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyModifiers},
    execute, queue,
    style::{Print, Stylize},
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, DisableLineWrap,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Progress {
    pub chapter: usize,
    pub line: usize,
}

pub fn run(epub: &mut Epub, progress: Option<Progress>) -> Result<Progress> {
    let (mut current_chapter, mut current_line) = if let Some(Progress { chapter, line }) = progress
    {
        (chapter, line)
    } else {
        (0, 0)
    };

    let mut text = epub.chapter(current_chapter)?;
    let mut stdout = std::io::stdout();

    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, DisableLineWrap, Hide)?;

    let (mut cols, mut rows) = size()?;

    loop {
        let indent = if cols > 80 { (cols - 80) / 2 } else { 0 };

        queue!(stdout, Clear(ClearType::All))?;

        for i in 0..rows {
            if let Some(line) = text.get(usize::from(i) + current_line) {
                queue!(stdout, MoveTo(indent, i), Print(line))?;
            }
        }

        let pages = text.len() / rows as usize + 1;
        let page = current_line / rows as usize + 1;
        let perc = (current_chapter as f32 / epub.len() as f32
            + (current_line as f32 / text.len() as f32) / epub.len() as f32)
            * 100.0;

        queue!(
            stdout,
            MoveTo(cols - 5, rows - 1),
            Print(format!("{:0>2}/{:0>2}", page, pages).bold()),
            MoveTo(cols - 5, 0),
            Print(format!(" {perc:>2.0}% ").bold().reverse()),
        )?;

        stdout.flush()?;

        let event = read()?;

        if let Event::Key(key) = event {
            use crossterm::event::KeyCode::*;
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
            {
                match key.code {
                    // Quit
                    Esc | Char('q') => break,
                    // Scroll down by a page
                    PageDown | Char(' ') => {
                        if text.len() - current_line > rows.into() {
                            current_line += usize::from(rows);
                        } else if current_chapter < epub.len() - 1 {
                            current_line = 0;
                            current_chapter += 1;
                            text = epub.chapter(current_chapter)?;
                        }
                    }
                    // Scroll up by a page
                    PageUp => {
                        if current_line >= rows as usize {
                            current_line -= rows as usize;
                        } else if current_line == 0 && current_chapter > 0 {
                            current_chapter -= 1;
                            text = epub.chapter(current_chapter)?;
                            current_line = ((text.len() - 1) / rows as usize) * rows as usize;
                        } else {
                            current_line = 0;
                        }
                    }
                    // Scroll down by a line
                    Down | Char('j') => {
                        if text.len() - 1 > current_line {
                            current_line += 1;
                        }
                    }
                    // Scroll up by a line
                    Up | Char('k') => {
                        if current_line > 0 {
                            current_line -= 1;
                        }
                    }
                    // Go to next chapter
                    Right | Char('l') => {
                        if current_chapter < epub.len() - 1 {
                            current_chapter += 1;
                            current_line = 0;
                            text = epub.chapter(current_chapter)?;
                        }
                    }
                    // Go to previous chapter
                    Left | Char('h') => {
                        if current_chapter > 0 {
                            current_chapter -= 1;
                            current_line = 0;
                            text = epub.chapter(current_chapter)?;
                        }
                    }
                    // Jump to start of chapter
                    Char('g') => {
                        current_line = 0;
                    }
                    // Jump to end of chapter
                    Char('G') => {
                        current_line = ((text.len() - 1) / rows as usize) * rows as usize;
                    }
                    _ => {}
                }
            }
        } else if let Event::Resize(x, y) = event {
            cols = x;
            rows = y;
        }
    }

    execute!(stdout, LeaveAlternateScreen, Show)?;
    disable_raw_mode()?;

    Ok(Progress {
        chapter: current_chapter,
        line: current_line,
    })
}
