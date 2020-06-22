use std::collections::HashMap;
use itertools::Itertools;

use crate::tasks::generator::*;
use crate::x;
use crate::dzen_format::DzenBuilder;

pub struct BarConfig {
    left: Vec<GenId>,
    right: Vec<GenId>,
    tray: bool,
    separator: String,
    padding: usize,
    split: f32,

    xinerama: usize,
    output: String,
    rect: x::Rectangle,
}

pub struct SetupConfig {
    arguments: HashMap<GenId, GenArg>,
    names: HashMap<GenId, String>,
    bars: Vec<BarConfig>,
    id: u8
}

impl SetupConfig {
    #![allow(dead_code)]
    pub fn new() -> Self {
        SetupConfig{
            arguments: HashMap::new(),
            names: HashMap::new(),
            bars: Vec::new(),
            id: 100 // TODO: count the elements in GenType
        }
    }

    fn create_module(&mut self, gen: GenType, arg: Option<GenArg>, name: Option<String>) -> GenId {
        if let Some(id) = self.module_exists(gen, &arg, &name) {
            return id;
        }

        if arg.is_some() || name.is_some() {
            let id = GenId::new(gen, self.id);
            self.id += 1;
            if let Some(a) = arg {
                self.arguments.insert(id, a);
            }
            if let Some(s) = name {
                self.name_module(id, s);
            }
            id
        } else {
            GenId::from_gen(gen)
        }
    }

    fn module_exists(&self, gen: GenType, arg: &Option<GenArg>, name: &Option<String>) -> Option<GenId> {
        for b in self.bars.iter() {
            for g in b.iter() {
                if g.gen == gen && self.names.get(g) == name.as_ref() && self.arguments.get(g) == arg.as_ref() {
                    return Some(*g);
                }
            }
        }
        None
    }

    pub fn name_module(&mut self, id: GenId, name: String) {
        self.names.insert(id, name);
    }

    pub fn add_bar(&mut self, bar: BarConfig) {
        self.bars.push(bar);
    }

    pub fn get_arg(&self, id: GenId) -> Option<&GenArg> {
        self.arguments.get(&id)
    }

    pub fn extract_args(&mut self) -> HashMap<GenId, GenArg> {
        self.arguments.drain().collect()
    }

    pub fn get_name(&self, id: GenId) -> Option<&String> {
        self.names.get(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item=&GenId> {
        self.bars.iter().flat_map(|b| b.iter()).unique()
    }

    pub fn take_bars(self) -> Vec<BarConfig> {
        self.bars
    }
}

impl BarConfig {
    #![allow(dead_code)]
    pub fn new(output: String, setup: &x::XSetup) -> Option<Self> {
        if let Some(xin) = setup.get_xinerama(&output) {
            if let Some(rect) = setup.get_rect(&output) {
                return Some(Self{
                    left: Vec::new(),
                    right: Vec::new(),
                    separator: " | ".to_string(),
                    tray: false,
                    padding: 10,
                    split: 0.5,
                    output: output,
                    xinerama: xin,
                    rect: rect
                });
            }
        }
        None
    }

    pub fn add_left(&mut self, id: GenId) {
        self.left.push(id);
    }

    pub fn add_right(&mut self, id: GenId) {
        self.right.push(id);
    }

    pub fn iter(&self) -> impl Iterator<Item=&GenId> {
        self.left.iter().chain(self.right.iter())
    }

    pub fn get_xinerama(&self) -> usize {
        self.xinerama
    }

    pub fn get_separator(&self) -> &str {
        &self.separator
    }

    pub fn iter_left(&self) -> impl Iterator<Item=&GenId> {
        self.left.iter()
    }

    pub fn iter_right(&self) -> impl Iterator<Item=&GenId> {
        self.right.iter()
    }

    pub fn get_output(&self) -> &str {
        &self.output
    }

    pub fn get_padding(&self) -> usize {
        self.padding
    }

    pub fn get_split(&self) -> f32 {
        self.split
    }

    pub fn get_screen_width(&self) -> u16 {
        self.rect.2
    }

    pub fn wants_tray(&self) -> bool {
        self.tray
    }
}

///////////////////////////////////////////////////////////////////////////////
//                                  builder                                  //
///////////////////////////////////////////////////////////////////////////////

pub type Result = std::result::Result<SetupConfig, Box<dyn std::error::Error>>;

pub struct SetupBuilder {
    bars: Vec<BarBuilder>,
    map_other: Option<Box<dyn Fn(String) -> BarBuilder>>,
    global_sep: Option<String>,
    global_pad: Option<usize>,
    global_split: Option<f32>
}

pub struct BarBuilder {
    output: String,
    left: Vec<GenBuilder>,
    right: Vec<GenBuilder>,
    tray: bool,
    sep: Option<String>,
    pad: Option<usize>,
    split: Option<f32>
}

pub struct GenBuilder {
    typ: GenType,
    name: Option<String>,
    arg: Option<String>,
    prepend: Option<DzenBuilder<'static>>,
    timeout: Option<u64>
}

impl SetupBuilder {
    #![allow(dead_code)]
    pub fn new() -> Self {
        SetupBuilder{
            bars: Vec::new(),
            map_other: None,
            global_sep: None,
            global_pad: None,
            global_split: None
        }
    }

    pub fn add_bar(mut self, bar: BarBuilder) -> Self {
        self.bars.push(bar);
        self
    }

    pub fn map_other<F>(mut self, f: F) -> Self
    where F: Fn(String) -> BarBuilder + 'static
    {
        self.map_other = Some(Box::new(f));
        self
    }

    pub fn separator<S: Into<String>>(mut self, sep: S) -> Self {
        self.global_sep = Some(sep.into());
        self
    }

    pub fn padding(mut self, pad: usize) -> Self {
        self.global_pad = Some(pad);
        self
    }

    pub fn split(mut self, split: f32) -> Self {
        self.global_split = Some(split);
        self
    }

    pub fn build(mut self) -> Result {
        let xsetup = x::get_x_setup()?;

        // TODO: handle mirroring of screens
        if let Some(f) = self.map_other {
            let used: Vec<_> = self.bars.iter()
                .map(|b| b.output.as_str())
                .collect();
            let unused: Vec<_> = xsetup.outputs()
                .filter(|o| !used.contains(o))
                .collect();
            for u in unused.into_iter() {
                self.bars.push(f(u.to_string()));
            }
        }

        let gsep = self.global_sep;
        let gpad = self.global_pad;
        let gsplit = self.global_split;
        let mut setup = SetupConfig::new();
        for b in self.bars.into_iter() {
            let mut bar = BarConfig::new(b.output, &xsetup).ok_or("output is not connected")?;
            bar.tray = b.tray;
            if let Some(sep) = b.sep.or_else(|| gsep.clone()) {
                bar.separator = sep;
            }
            if let Some(pad) = b.pad.or_else(|| gpad) {
                bar.padding = pad;
            }
            if let Some(split) = b.split.or_else(|| gsplit) {
                bar.split = split;
            }

            SetupBuilder::build_side(b.left, &mut setup, |id| bar.add_left(id));
            SetupBuilder::build_side(b.right, &mut setup, |id| bar.add_right(id));
            setup.add_bar(bar);
        }

        Ok(setup)
    }

    fn build_side<F>(gens: Vec<GenBuilder>, setup: &mut SetupConfig, mut bar_add: F)
    where F: FnMut(GenId)
    {
        for l in gens.into_iter() {
            let args = if l.timeout.is_none() && l.arg.is_none() && l.prepend.is_none() {
                None
            } else {
                Some(GenArg{timeout: l.timeout, arg: l.arg, prepend: l.prepend})
            };
            let id = setup.create_module(l.typ, args, l.name);
            bar_add(id);
        }

    }
}

impl BarBuilder {
    #![allow(dead_code)]
    pub fn new<S: Into<String>>(output: S) -> Self {
        BarBuilder{
            output: output.into(),
            left: Vec::new(),
            right: Vec::new(),
            tray: false,
            sep: None,
            pad: None,
            split: None
        }
    }

    pub fn add_left(mut self, gen: GenBuilder) -> Self {
        self.left.push(gen);
        self
    }

    pub fn add_right(mut self, gen: GenBuilder) -> Self {
        self.right.push(gen);
        self
    }

    pub fn tray(mut self, t: bool) -> Self {
        self.tray = t;
        self
    }

    pub fn separator<S: Into<String>>(mut self, sep: S) -> Self {
        self.sep = Some(sep.into());
        self
    }

    pub fn padding(mut self, pad: usize) -> Self {
        self.pad = Some(pad);
        self
    }

    pub fn split(mut self, split: f32) -> Self {
        self.split = Some(split);
        self
    }
}

impl GenBuilder {
    #![allow(dead_code)]
    pub fn new(typ: GenType) -> Self {
        GenBuilder{
            typ: typ,
            name: None,
            arg: None,
            prepend: None,
            timeout: None
        }
    }

    pub fn name<S: Into<String>>(mut self, n: S) -> Self {
        self.name = Some(n.into());
        self
    }

    pub fn argument<S: Into<String>>(mut self, arg: S) -> Self {
        self.arg = Some(arg.into());
        self
    }

    pub fn prepend(mut self, pre: DzenBuilder<'static>) -> Self {
        self.prepend = Some(pre);
        self
    }

    pub fn timeout(mut self, tim: u64) -> Self {
        self.timeout = Some(tim);
        self
    }

}
