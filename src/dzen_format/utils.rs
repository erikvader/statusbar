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

pub fn bytes_to_ibibyte_string(b: u64) -> String {
    let mag = if b == 0 {
        0 as f64
    } else {
        (b as f64).log2() / (1024 as f64).log2()
    }.floor();

    let mut num = b as f64 / 1024_f64.powf(mag);
    num *= 10_f64;
    num = num.trunc();
    num /= 10_f64;

    let unit = match mag as u32 {
        0 => "B",
        1 => "KiB",
        2 => "MiB",
        3 => "GiB",
        4 => "TiB",
        5 => "PiB",
        6 => "EiB",
        7 => "ZiB",
        _ => "YiB"
    };
    format!("{:.1} {}", num, unit)
}
