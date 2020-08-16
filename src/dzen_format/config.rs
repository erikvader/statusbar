use std::collections::HashMap;

#[derive(Clone,PartialEq,Debug)]
pub struct Config<'a> {
    pub color: HashMap<&'a str, &'a str>,
    pub icon:  HashMap<&'a str, &'a str>,
}

impl<'a> Config<'a> {
    pub fn new() -> Self {
        Config {
            color: HashMap::new(),
            icon: HashMap::new(),
        }
    }
}
