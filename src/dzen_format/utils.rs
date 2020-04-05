use super::*;
use crate::config::*;

impl<'a> DzenBuilder<'a> {
    pub fn name_click(self, button: &'a str, module_name: &'a str) -> Self {
        self.click(button, &["echo ", module_name, " click ", button, " >> ", FIFO_PATH])
    }

    pub fn color_step(self, steps: &[(i32, &'a str)]) -> Self {
        let inner: i32 = if let Ok(n) = self.to_string().parse() {
            n
        } else {
            eprintln!("couldn't parse as number");
            return self;
        };

        let mut color = None;
        for (lim, col) in steps.iter() {
            if inner >= *lim {
                color = Some(col);
            } else {
                break;
            }
        }

        if let Some(col) = color {
            self.colorize(col)
        } else {
            self
        }
    }
}
