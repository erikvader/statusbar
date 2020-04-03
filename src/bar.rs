use std::collections::HashMap;
use itertools::Itertools;

use crate::tasks::generator::*;
use crate::x;

pub struct BarConfig {
    left: Vec<GenId>,
    right: Vec<GenId>,
    tray: bool,
    separator: String,
    padding: usize,

    xinerama: usize,
    output: String,
    rect: x::Rectangle,
}

pub struct SetupConfig {
    arguments: HashMap<GenId, String>,
    names: HashMap<GenId, String>,
    bars: Vec<BarConfig>,
    id: u8
}

impl SetupConfig {
    pub fn new() -> Self {
        SetupConfig{
            arguments: HashMap::new(),
            names: HashMap::new(),
            bars: Vec::new(),
            id: 100 // TODO: count the elements in GenType
        }
    }

    pub fn create_module(&mut self, gen: GenType, arg: Option<String>) -> GenId {
        if let Some(a) = arg {
            let id = GenId::new(gen, self.id);
            self.id += 1;
            self.arguments.insert(id, a);
            id
        } else {
            GenId::from_gen(gen)
        }
    }

    pub fn name_module(&mut self, id: GenId, name: String) {
        self.names.insert(id, name);
    }

    pub fn add_bar(&mut self, bar: BarConfig) {
        self.bars.push(bar);
    }

    pub fn get_arg(&self, id: GenId) -> Option<&String> {
        self.arguments.get(&id)
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
    pub fn new(output: String, setup: &x::XSetup) -> Option<Self> {
        if let Some(xin) = setup.get_xinerama(&output) {
            if let Some(rect) = setup.get_rect(&output) {
                return Some(Self{
                    left: Vec::new(),
                    right: Vec::new(),
                    separator: " | ".to_string(),
                    tray: false,
                    padding: 10,
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

    pub fn get_screen_width(&self) -> u16 {
        self.rect.2
    }
}
