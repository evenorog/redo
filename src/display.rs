use crate::{At, Command, Entry, History, Record, Signal};
use alloc::format;
use alloc::string::ToString;
#[cfg(feature = "chrono")]
use chrono::{DateTime, Local, Utc};
use colored::{Color, Colorize};
use core::fmt::{self, Write};

/// Configurable display formatting of structures.
///
/// # Examples
/// ```
/// # use std::fmt::{self, Display, Formatter};
/// # use redo::{Command, History};
/// # struct Add(char);
/// # impl Command for Add {
/// #     type Target = String;
/// #     type Error = ();
/// #     fn apply(&mut self, s: &mut String) -> redo::Result<Add> { Ok(()) }
/// #     fn undo(&mut self, s: &mut String) -> redo::Result<Add> { Ok(()) }
/// # }
/// # impl Display for Add {
/// #     fn fmt(&self, f: &mut Formatter) -> fmt::Result { Ok(()) }
/// # }
/// # fn foo() -> History<Add> {
/// let history = History::default();
/// println!(
///     "{}",
///     history.display().graph(true).colored(true).unicode(true)
/// );
/// # history
/// # }
/// ```
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Display<'a, T> {
    data: &'a T,
    view: View,
}

impl<T> Display<'_, T> {
    /// Show colored output (off by default).
    #[inline]
    pub fn colored(&mut self, on: bool) -> &mut Self {
        self.view.colored = on;
        self
    }

    /// Show the current position in the output (on by default).
    #[inline]
    pub fn current(&mut self, on: bool) -> &mut Self {
        self.view.current = on;
        self
    }

    /// Show detailed output (on by default).
    #[inline]
    pub fn detailed(&mut self, on: bool) -> &mut Self {
        self.view.detailed = on;
        self
    }

    /// Use unicode symbols in the output (off by default).
    ///
    /// The symbols might only work as expected with monospaced fonts.
    #[inline]
    pub fn unicode(&mut self, on: bool) -> &mut Self {
        self.view.unicode = on;
        self
    }

    /// Show the position of the command (on by default).
    #[inline]
    pub fn position(&mut self, on: bool) -> &mut Self {
        self.view.position = on;
        self
    }

    /// Show the saved command (on by default).
    #[inline]
    pub fn saved(&mut self, on: bool) -> &mut Self {
        self.view.saved = on;
        self
    }
}

impl<C: Command, F> Display<'_, History<C, F>> {
    /// Show the history as a graph (off by default).
    #[inline]
    pub fn graph(&mut self, on: bool) -> &mut Self {
        self.view.graph = on;
        self
    }
}

impl<C: Command + fmt::Display, F: FnMut(Signal)> Display<'_, Record<C, F>> {
    #[inline]
    fn fmt_list(&self, f: &mut fmt::Formatter, at: At, entry: &Entry<C>) -> fmt::Result {
        self.view.mark(f, 0)?;
        self.view.position(f, at, false)?;
        if self.view.detailed {
            #[cfg(feature = "chrono")]
            self.view.timestamp(f, &entry.timestamp)?;
        }
        self.view.current(
            f,
            at,
            At {
                branch: 0,
                current: self.data.current(),
            },
        )?;
        self.view.saved(
            f,
            at,
            self.data.saved.map(|saved| At {
                branch: 0,
                current: saved,
            }),
        )?;
        if self.view.detailed {
            writeln!(f)?;
            self.view.message(f, entry, 0)
        } else {
            f.write_char(' ')?;
            self.view.message(f, entry, 0)?;
            writeln!(f)
        }
    }
}

impl<C: Command + fmt::Display, F: FnMut(Signal)> Display<'_, History<C, F>> {
    #[inline]
    fn fmt_list(
        &self,
        f: &mut fmt::Formatter,
        at: At,
        entry: &Entry<C>,
        level: usize,
    ) -> fmt::Result {
        self.view.mark(f, level)?;
        self.view.position(f, at, true)?;
        if self.view.detailed {
            #[cfg(feature = "chrono")]
            self.view.timestamp(f, &entry.timestamp)?;
        }
        self.view.current(
            f,
            at,
            At {
                branch: self.data.branch(),
                current: self.data.current(),
            },
        )?;
        self.view.saved(
            f,
            at,
            self.data
                .record
                .saved
                .map(|saved| At {
                    branch: self.data.branch(),
                    current: saved,
                })
                .or(self.data.saved),
        )?;
        if self.view.detailed {
            writeln!(f)?;
            self.view.message(f, entry, level)
        } else {
            f.write_char(' ')?;
            self.view.message(f, entry, level)?;
            writeln!(f)
        }
    }

    #[inline]
    fn fmt_graph(
        &self,
        f: &mut fmt::Formatter,
        at: At,
        entry: &Entry<C>,
        level: usize,
    ) -> fmt::Result {
        for (&i, branch) in self
            .data
            .branches
            .iter()
            .filter(|(_, branch)| branch.parent == at)
        {
            for (j, cmd) in branch.commands.iter().enumerate().rev() {
                let at = At {
                    branch: i,
                    current: j + branch.parent.current + 1,
                };
                self.fmt_graph(f, at, cmd, level + 1)?;
            }
            for j in 0..level {
                self.view.edge(f, j)?;
                f.write_char(' ')?;
            }
            self.view.split(f, level)?;
            writeln!(f)?;
        }
        for i in 0..level {
            self.view.edge(f, i)?;
            f.write_char(' ')?;
        }
        self.fmt_list(f, at, entry, level)
    }
}

impl<'a, T> From<&'a T> for Display<'a, T> {
    #[inline]
    fn from(data: &'a T) -> Self {
        Display {
            data,
            view: View::default(),
        }
    }
}

impl<C: Command + fmt::Display, F: FnMut(Signal)> fmt::Display for Display<'_, Record<C, F>> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, cmd) in self.data.commands.iter().enumerate().rev() {
            let at = At {
                branch: 0,
                current: i + 1,
            };
            self.fmt_list(f, at, cmd)?;
        }
        Ok(())
    }
}

impl<C: Command, F: FnMut(Signal)> fmt::Display for Display<'_, History<C, F>>
where
    C: fmt::Display,
    C::Target: fmt::Display,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, cmd) in self.data.record.commands.iter().enumerate().rev() {
            let at = At {
                branch: self.data.branch(),
                current: i + 1,
            };
            if self.view.graph {
                self.fmt_graph(f, at, cmd, 0)?;
            } else {
                self.fmt_list(f, at, cmd, 0)?;
            }
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
struct View {
    colored: bool,
    current: bool,
    detailed: bool,
    graph: bool,
    unicode: bool,
    position: bool,
    saved: bool,
}

impl Default for View {
    #[inline]
    fn default() -> Self {
        View {
            colored: false,
            current: true,
            detailed: true,
            graph: false,
            unicode: false,
            position: true,
            saved: true,
        }
    }
}

impl View {
    #[inline]
    fn message(self, f: &mut fmt::Formatter, msg: &impl ToString, level: usize) -> fmt::Result {
        let msg = msg.to_string();
        let lines = msg.lines();
        if self.detailed {
            for line in lines {
                for i in 0..=level {
                    self.edge(f, i)?;
                    f.write_char(' ')?;
                }
                writeln!(f, "{}", line.trim())?;
            }
        } else if let Some(line) = lines.map(str::trim).find(|s| !s.is_empty()) {
            f.write_str(&line)?;
        }
        Ok(())
    }

    #[inline]
    fn mark(self, f: &mut fmt::Formatter, level: usize) -> fmt::Result {
        match (self.colored, self.unicode) {
            (true, true) => write!(f, "{}", "\u{25CF}".color(to_color(level))),
            (true, false) => write!(f, "{}", "*".color(to_color(level))),
            (false, true) => f.write_char('\u{25CF}'),
            (false, false) => f.write_char('*'),
        }
    }

    #[inline]
    fn edge(self, f: &mut fmt::Formatter, level: usize) -> fmt::Result {
        match (self.colored, self.unicode) {
            (true, true) => write!(f, "{}", "\u{2502}".color(to_color(level))),
            (true, false) => write!(f, "{}", "|".color(to_color(level))),
            (false, true) => f.write_char('\u{2502}'),
            (false, false) => f.write_char('|'),
        }
    }

    #[inline]
    fn split(self, f: &mut fmt::Formatter, level: usize) -> fmt::Result {
        match (self.colored, self.unicode) {
            (true, true) => write!(
                f,
                "{}{}{}",
                "\u{251C}".color(to_color(level)),
                "\u{2500}".color(to_color(level + 1)),
                "\u{256F}".color(to_color(level + 1))
            ),
            (true, false) => write!(
                f,
                "{}{}",
                "|".color(to_color(level)),
                "/".color(to_color(level + 1))
            ),
            (false, true) => f.write_str("\u{251C}\u{2500}\u{256F}"),
            (false, false) => f.write_str("|/"),
        }
    }

    #[inline]
    fn position(self, f: &mut fmt::Formatter, at: At, use_branch: bool) -> fmt::Result {
        if self.position {
            if self.colored {
                let position = if use_branch {
                    format!("[{}:{}]", at.branch, at.current)
                } else {
                    format!("[{}]", at.current)
                };
                write!(f, " {}", position.yellow())
            } else if use_branch {
                write!(f, " [{}:{}]", at.branch, at.current)
            } else {
                write!(f, " [{}]", at.current)
            }
        } else {
            Ok(())
        }
    }

    #[inline]
    fn current(self, f: &mut fmt::Formatter, at: At, current: At) -> fmt::Result {
        if self.current && at == current {
            if self.colored {
                write!(f, " {}{}{}", "(".yellow(), "current".cyan(), ")".yellow())
            } else {
                f.write_str(" (current)")
            }
        } else {
            Ok(())
        }
    }

    #[inline]
    fn saved(self, f: &mut fmt::Formatter, at: At, saved: Option<At>) -> fmt::Result {
        if self.saved && saved.map_or(false, |saved| saved == at) {
            if self.colored {
                write!(
                    f,
                    " {}{}{}",
                    "(".yellow(),
                    "saved".bright_green(),
                    ")".yellow()
                )
            } else {
                f.write_str(" (saved)")
            }
        } else {
            Ok(())
        }
    }

    #[inline]
    #[cfg(feature = "chrono")]
    fn timestamp(self, f: &mut fmt::Formatter, timestamp: &DateTime<Utc>) -> fmt::Result {
        let rfc2822 = timestamp.with_timezone(&Local).to_rfc2822();
        if self.colored {
            write!(f, " {}{}{}", "[".yellow(), rfc2822.yellow(), "]".yellow())
        } else {
            write!(f, " [{}]", rfc2822)
        }
    }
}

#[inline]
fn to_color(i: usize) -> Color {
    match i % 6 {
        0 => Color::Cyan,
        1 => Color::Red,
        2 => Color::Magenta,
        3 => Color::Yellow,
        4 => Color::Green,
        5 => Color::Blue,
        _ => unreachable!(),
    }
}
