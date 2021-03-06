pub mod utils;
pub mod parser;
pub mod external;
pub mod config;

use std::collections::VecDeque;
use std::ops::{Add,Rem};
use std::fmt;
use std::borrow::Cow;
use config::Config;

#[derive(Clone,PartialEq,Eq,Debug)]
pub struct DzenBuilder<'a> {
    theme: Option<&'a Config<'a>>,
    work: VecDeque<Cow<'a, str>>,
    res: Vec<Cow<'a, str>>
}

impl<'a> DzenBuilder<'a> {
    #![allow(dead_code)]
    // creation ///////////////////////////////////////////////////////////////
    pub fn new() -> Self {
        DzenBuilder{
            theme: None,
            work: VecDeque::new(),
            res: Vec::new()
        }
    }

    pub fn from_str<S>(s: S) -> Self
    where S: Into<Cow<'a, str>>
    {
        Self::new().add(s)
    }

    pub fn to_stringln(self) -> String {
        self.add("\n").to_string()
    }

    pub fn use_theme(mut self, theme: &'a Config<'a>) -> Self {
        self.theme = Some(theme);
        self
    }

    // sections ///////////////////////////////////////////////////////////////
    pub fn new_section(mut self) -> Self {
        for w in self.work.drain(..) {
            self.res.push(w);
        }
        self.work.clear();
        self
    }

    pub fn everything(mut self) -> Self {
        for r in self.res.drain(..).rev() {
            self.work.push_front(r);
        }
        self.res.clear();
        self
    }

    // adapters ///////////////////////////////////////////////////////////////
    pub fn append_icon<S>(self, icon: S) -> Self
    where S: Into<Cow<'a, str>>
    {
        let asd = self.icon_strs(icon.into());
        asd.into_iter().fold(self, |s, a| s.add(a))
    }

    pub fn prepend_icon<S>(self, icon: S) -> Self
    where S: Into<Cow<'a, str>>
    {
        let asd = self.icon_strs(icon.into());
        asd.into_iter().rev().fold(self, |s, a| s.pre(a))
    }

    pub fn colorize<S>(self, color: S) -> Self
    where S: Into<Cow<'a, str>>
    {
        let c = color.into();
        let col = self.theme
            .and_then(|m| m.color.get(c.as_ref()))
            .map_or_else(|| c, |s| Cow::from(*s));

        self.add("^fg()")
            .pre(")")
            .pre(col)
            .pre("^fg(")
    }

    pub fn background<S>(self, color: S) -> Self
    where S: Into<Cow<'a, str>>
    {
        let c = color.into();
        let col = self.theme
            .and_then(|m| m.color.get(c.as_ref()))
            .map_or_else(|| c, |s| Cow::from(*s));

        self.add("^bg()")
            .pre(")")
            .pre(col)
            .pre("^bg(")
    }

    pub fn click<S>(self, button: usize, command: S) -> Self
    where S: Into<Cow<'a, str>>
    {
        self.add("^ca()")
            .pre(")")
            .pre(command)
            .pre(", ")
            .pre(button.to_string())
            .pre("^ca(")
    }

    pub fn position(self, x: isize, y: isize) -> Self {
        self.pre(")")
            .pre(y.to_string())
            .pre(";")
            .pre(x.to_string())
            .pre("^pa(")
    }

    pub fn position_x(self, x: isize) -> Self {
        self.pre(")")
            .pre(x.to_string())
            .pre("^pa(")
    }

    pub fn shift(self, x: isize, y: isize) -> Self {
        self.pre(")")
            .pre(y.to_string())
            .pre(";")
            .pre(x.to_string())
            .pre("^p(")
    }

    pub fn lpad(self, x: usize) -> Self {
        self.pre(")")
            .pre(x.to_string())
            .pre("^p(")
    }

    pub fn rpad(self, x: usize) -> Self
    {
        self.add("^p(")
            .add(x.to_string())
            .add(")")
    }

    // NOTE: Only works if self doesn't contain any tags. bug(?) in dzen
    // pub fn block_align<S>(self, width: S, align: S) -> Self
    // where S: Into<Cow<'a, str>>
    // {
    //     self.surround(&["^ba(", width, ",", align, ")"], &[])
    // }

    pub fn add<S>(mut self, s: S) -> Self
    where S: Into<Cow<'a, str>>
    {
        self.work.push_back(s.into());
        self
    }

    pub fn pre<S>(mut self, s: S) -> Self
    where S: Into<Cow<'a, str>>
    {
        self.work.push_front(s.into());
        self
    }

    pub fn add_not_empty<S>(self, s: S) -> Self
    where S: Into<Cow<'a, str>>
    {
        let e = !self.work.is_empty();
        self.maybe_add(e, s)
    }

    pub fn guard<F>(self, b: bool, f: F) -> Self
    where F: FnOnce(Self) -> Self
    {
        if b {
            f(self)
        } else {
            self
        }
    }

    pub fn maybe_add<S>(self, b: bool, s: S) -> Self
    where S: Into<Cow<'a, str>>
    {
        self.guard(b, |sel| sel.add(s))
    }

    pub fn rect(self, width: usize, height: usize) -> Self {
        self.add("^r(")
            .add(width.to_string())
            .add("x")
            .add(height.to_string())
            .add(")")
    }

    fn icon_strs(&self, icon: Cow<'a, str>) -> Vec<Cow<'a, str>> {
        let mut tmp: Vec<Cow<'a, str>> = vec!["^i(".into()];
        let path = crate::config::ICON_PATH;
        if path.starts_with("~") {
            let h = std::env::var("HOME").expect("couldn't get HOME");
            tmp.push(h.into());
            tmp.push(path[1..].into());
        } else {
            tmp.push(path.into());
        }
        tmp.push("/".into());

        let ico = self.theme
            .and_then(|m| m.icon.get(icon.as_ref()))
            .map_or_else(|| icon, |s| Cow::from(*s));
        tmp.push(ico);

        tmp.push(".xpm".into());
        tmp.push(")".into());
        tmp
    }
}

impl fmt::Display for DzenBuilder<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for s in self.res.iter() {
            f.write_str(s)?;
        }
        for s in self.work.iter() {
            f.write_str(s)?;
        }
        Ok(())
    }
}

impl<'a> Add<&'a str> for DzenBuilder<'a> {
    type Output = Self;
    fn add(self, other: &'a str) -> Self::Output {
        self.add(other)
    }
}

impl Add<String> for DzenBuilder<'_> {
    type Output = Self;
    fn add(self, other: String) -> Self::Output {
        self.add(other)
    }
}

impl<'a> Rem<&'a str> for DzenBuilder<'a> {
    type Output = Self;
    fn rem(self, other: &'a str) -> Self::Output {
        self.add_not_empty(other)
    }
}

impl Rem<String> for DzenBuilder<'_> {
    type Output = Self;
    fn rem(self, other: String) -> Self::Output {
        self.add_not_empty(other)
    }
}

impl<'a> From<&'a str> for DzenBuilder<'a> {
    fn from(s: &'a str) -> Self {
        Self::from_str(s)
    }
}

impl From<String> for DzenBuilder<'_> {
    fn from(s: String) -> Self {
        Self::from_str(s)
    }
}
