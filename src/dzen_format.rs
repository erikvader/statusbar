pub mod utils;

use std::collections::VecDeque;
use std::ops::Add;
use std::fmt;

pub struct DzenBuilder<'a> {
    work: VecDeque<&'a str>
}

impl<'a> DzenBuilder<'a> {
    // creation ///////////////////////////////////////////////////////////////
    fn new() -> Self {
        DzenBuilder{
            work: VecDeque::new()
        }
    }

    pub fn from_icon(path: &'a str) -> Self {
        Self::from_str(path)
            .surround(&["^i("], &[")"])
    }

    pub fn from_str(s: &'a str) -> Self {
        Self::new().push(s)
    }

    // adapters ///////////////////////////////////////////////////////////////
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

    // helpers ////////////////////////////////////////////////////////////////
    fn push(mut self, s: &'a str) -> Self {
        self.work.push_back(s);
        self
    }

    fn surround(mut self, before: &[&'a str], after: &[&'a str]) -> Self {
        for s in before.iter().rev() {
            self.work.push_front(s);
        }
        for s in after {
            self.work.push_back(s);
        }
        self
    }
}

impl fmt::Display for DzenBuilder<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for s in self.work.iter() {
            f.write_str(s)?;
        }
        Ok(())
    }
}

impl<'a> Add for DzenBuilder<'a> {
    type Output = Self;
    fn add(mut self, mut other: Self) -> Self::Output {
        self.work.append(&mut other.work);
        self
    }
}

impl<'a> Add<&'a str> for DzenBuilder<'a> {
    type Output = Self;
    fn add(self, other: &'a str) -> Self::Output {
        self.push(other)
    }
}

impl<'a> From<&'a str> for DzenBuilder<'a> {
    fn from(s: &'a str) -> Self {
        Self::from_str(s)
    }
}
