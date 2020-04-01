mod utils;

use std::collections::VecDeque;
use std::ops::Add;

// general traits /////////////////////////////////////////////////////////////
pub trait DzenFormat<'a> {
    fn format(&self, work: &mut VecDeque<&'a str>);

    // convert to string
    fn to_string(&self) -> String {
        let mut work = VecDeque::new();
        self.format(&mut work);
        work.into_iter().collect()
    }

    // adapters ///////////////////////////////////////////////////////////////
    fn colorize(self, color: &'a str) -> Colorize<'a, Self>
    where Self: Sized
    {
        Colorize{inner: self, color: color}
    }

    fn background(self, color: &'a str) -> Background<'a, Self>
    where Self: Sized
    {
        Background{inner: self, color: color}
    }

    fn concat<O>(self, other: O) -> Concat<Self, O>
    where O: DzenFormat<'a>,
          Self: Sized
    {
        Concat{left: self, right: other}
    }

    fn id(self) -> Id<Self>
    where Self: Sized
    {
        Id(self)
    }

    fn click(self, button: &'a str, command: &'a str) -> Click<'a, Self>
    where Self: Sized
    {
        Click{inner: self, button, command}
    }
}

// string conversions /////////////////////////////////////////////////////////
impl<'a> DzenFormat<'a> for &'a str {
    fn format(&self, work: &mut VecDeque<&'a str>) {
        work.push_back(self)
    }
}

// colorize ///////////////////////////////////////////////////////////////////
pub struct Colorize<'a, D> {
    inner: D,
    color: &'a str
}

impl<'a, D> DzenFormat<'a> for Colorize<'a, D>
where D: DzenFormat<'a>
{
    fn format(&self, work: &mut VecDeque<&'a str>) {
        self.inner.format(work);
        surround(&["^fg(", self.color, ")"], &["^fg()"], work);
    }
}

pub struct Background<'a, D> {
    inner: D,
    color: &'a str
}

impl<'a, D> DzenFormat<'a> for Background<'a, D>
where D: DzenFormat<'a>
{
    fn format(&self, work: &mut VecDeque<&'a str>) {
        self.inner.format(work);
        surround(&["^bg(", self.color, ")"], &["^bg()"], work);
    }
}

// id /////////////////////////////////////////////////////////////////////////
pub struct Id<T>(T);

impl<'a,T> DzenFormat<'a> for Id<T>
where T: DzenFormat<'a>
{
    fn format(&self, work: &mut VecDeque<&'a str>) {
        self.0.format(work)
    }
}

// empty //////////////////////////////////////////////////////////////////////
pub struct Empty;

impl DzenFormat<'_> for Empty {
    fn format(&self, _work: &mut VecDeque<&'_ str>) {}
}

pub fn id() -> Id<Empty> {
    Id(Empty)
}

// icon ///////////////////////////////////////////////////////////////////////
pub struct Icon<'a> {
    path: &'a str
}

impl<'a> DzenFormat<'a> for Icon<'a> {
    fn format(&self, work: &mut VecDeque<&'a str>) {
        surround(&[], &["^i(", self.path, ")"], work);
    }
}

pub fn icon<'a>(path: &'a str) -> Icon<'a> {
    Icon{path}
}

// space //////////////////////////////////////////////////////////////////////
pub struct Spacing<'a> {
    pixels: &'a str
}

impl<'a> DzenFormat<'a> for Spacing<'a> {
    fn format(&self, work: &mut VecDeque<&'a str>) {
        surround(&[], &["^p(+", self.pixels, ")"], work);
    }
}

pub fn spacing<'a>(pixels: &'a str) -> Spacing<'a> {
    Spacing{pixels}
}

// click //////////////////////////////////////////////////////////////////////
pub struct Click<'a,D> {
    button: &'a str,
    command: &'a str,
    inner: D
}

impl<'a,D> DzenFormat<'a> for Click<'a,D>
where D: DzenFormat<'a>
{
    fn format(&self, work: &mut VecDeque<&'a str>) {
        self.inner.format(work);
        surround(&["^ca(", self.button, ", ", self.command, ")"], &["^ca()"], work);
    }
}

// concat /////////////////////////////////////////////////////////////////////
pub struct Concat<A, B> {
    left: A,
    right: B
}

impl<'a, A, B> DzenFormat<'a> for Concat<A, B>
where A: DzenFormat<'a>,
      B: DzenFormat<'a>
{
    fn format(&self, work: &mut VecDeque<&'a str>) {
        let mut tmp = VecDeque::new();
        self.left.format(&mut tmp);
        // NOTE: append moves all elements from tmp to work
        work.append(&mut tmp);
        self.right.format(&mut tmp);
        work.append(&mut tmp);
    }
}

impl<'a,T,S> Add<S> for Id<T>
where T: DzenFormat<'a>,
      S: DzenFormat<'a>
{
    type Output = Id<Concat<T, S>>;
    fn add(self, other: S) -> Self::Output {
        Id(self.0.concat(other))
    }
}

// helpers ////////////////////////////////////////////////////////////////////
fn surround<'a>(before: &[&'a str], after: &[&'a str], work: &mut VecDeque<&'a str>) {
    for s in before.iter().rev() {
        work.push_front(s);
    }
    for s in after {
        work.push_back(s);
    }
}
