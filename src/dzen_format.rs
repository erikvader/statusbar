pub mod utils;

use std::collections::VecDeque;
use std::ops::{Add,Rem};
use std::fmt;

pub struct DzenBuilder<'a> {
    work: VecDeque<&'a str>,
    res: String
}

impl<'a> DzenBuilder<'a> {
    // creation ///////////////////////////////////////////////////////////////
    pub fn new() -> Self {
        DzenBuilder{
            work: VecDeque::new(),
            res: String::new()
        }
    }

    pub fn from_str(s: &'a str) -> Self {
        Self::new().add(s)
    }

    pub fn to_stringln(self) -> String {
        self.add("\n").to_string()
    }

    // adapters ///////////////////////////////////////////////////////////////
    pub fn new_section(mut self) -> Self {
        for w in self.work.iter() {
            self.res = self.res + w;
        }
        self.work.clear();
        self
    }

    pub fn append_icon(self, icon: &'a str) -> Self {
        let asd = DzenBuilder::icon_strs(icon);
        self.surround(&[], &asd)
    }

    pub fn prepend_icon(self, icon: &'a str) -> Self {
        let asd = DzenBuilder::icon_strs(icon);
        self.surround(&asd, &[])
    }

    pub fn colorize(self, color: &'a str) -> Self {
        self.surround(&["^fg(", color, ")"], &["^fg()"])
    }

    pub fn background(self, color: &'a str) -> Self {
        self.surround(&["^bg(", color, ")"], &["^bg()"])
    }

    pub fn click(self, button: &'a str, command: &[&'a str]) -> Self {
        self.surround(&[")"], &["^ca()"])
            .surround(command, &[])
            .surround(&["^ca(", button, ", "], &[])
    }

    pub fn position(self, x: &'a str, y: &'a str) -> Self {
        self.surround(&["^pa(", x, ";", y, ")"], &[])
    }

    pub fn position_x(self, x: &'a str) -> Self {
        self.position(x, "")
    }

    pub fn shift(self, x: &'a str, y: &'a str) -> Self {
        self.surround(&["^p(", x, ";", y, ")"], &[])
    }

    pub fn lpad(self, x: &'a str) -> Self {
        self.shift(x, "")
    }

    pub fn rpad(self, x: &'a str) -> Self {
        self.surround(&[], &["^p(", x, ")"])
    }

    // NOTE: Only works if self doesn't contain any tags. bug(?) in dzen
    pub fn block_align(self, width: &'a str, align: &'a str) -> Self {
        self.surround(&["^ba(", width, ",", align, ")"], &[])
    }

    pub fn add(mut self, s: &'a str) -> Self {
        self.work.push_back(s);
        self
    }

    pub fn add_not_empty(self, s: &'a str) -> Self {
        let e = !self.work.is_empty();
        self.maybe_add(e, s)
    }

    pub fn maybe_add(self, b: bool, s: &'a str) -> Self {
        if b {
            self.add(s)
        } else {
            self
        }
    }

    // helpers ////////////////////////////////////////////////////////////////
    fn surround(mut self, before: &[&'a str], after: &[&'a str]) -> Self {
        for s in before.iter().rev() {
            self.work.push_front(s);
        }
        for s in after {
            self.work.push_back(s);
        }
        self
    }

    fn icon_strs(icon: &'a str) -> Vec<&'a str> {
        let mut tmp = vec!["^i("];
        let path = crate::config::ICON_PATH;
        if path.starts_with("~") {
            let h = unsafe{crate::HOME.as_str()};
            tmp.push(h);
            tmp.push(&path[1..]);
        } else {
            tmp.push(path);
        }
        tmp.push("/");
        tmp.push(icon);
        tmp.push(")");
        tmp
    }
}

impl fmt::Display for DzenBuilder<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.res)?;
        for s in self.work.iter() {
            f.write_str(s)?;
        }
        Ok(())
    }
}

// impl<'a> Add for DzenBuilder<'a> {
//     type Output = Self;
//     fn add(mut self, mut other: Self) -> Self::Output {
//         self.work.append(&mut other.work);
//         self
//     }
// }

impl<'a> Add<&'a str> for DzenBuilder<'a> {
    type Output = Self;
    fn add(self, other: &'a str) -> Self::Output {
        self.add(other)
    }
}

impl<'a> Rem<&'a str> for DzenBuilder<'a> {
    type Output = Self;
    fn rem(self, other: &'a str) -> Self::Output {
        self.add_not_empty(other)
    }
}

impl<'a> From<&'a str> for DzenBuilder<'a> {
    fn from(s: &'a str) -> Self {
        Self::from_str(s)
    }
}

impl<'a> From<&'a String> for DzenBuilder<'a> {
    fn from(s: &'a String) -> Self {
        Self::from_str(s.as_str())
    }
}
