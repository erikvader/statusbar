use super::*;
use crate::constants::*;

impl<'a> DzenBuilder<'a> {
    pub fn self_click(self, button: &'a str, module_name: &'a str) -> Self {
        self.click(button, &["echo ", module_name, " click ", button, " >> ", FIFO_PATH])
    }
}
