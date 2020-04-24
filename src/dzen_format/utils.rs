use super::*;
use crate::config::*;

impl<'a> DzenBuilder<'a> {
    pub fn name_click(self, button: usize, module_name: impl AsRef<str>) -> Self {
        self.click(button, format!("echo {} click {} >> {}",
                                   module_name.as_ref(),
                                   button,
                                   FIFO_PATH))
    }

    pub fn push_color_step(self, num: i32, steps: &[(i32, &'a str)]) -> Self {
        let mut color = None;
        for (lim, col) in steps.iter() {
            if num >= *lim {
                color = Some(col);
            } else {
                break;
            }
        }

        let tmp = self.new_section().add(num.to_string());
        if let Some(col) = color {
            tmp.colorize(*col)
        } else {
            tmp
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
