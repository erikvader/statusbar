use super::*;
use crate::config::*;

impl<'a> DzenBuilder<'a> {
    pub fn name_click(self, button: usize, module_name: impl AsRef<str>) -> Self {
        self.click(button, format!("echo {} click {} >> {}",
                                   module_name.as_ref(),
                                   button,
                                   FIFO_PATH))
    }

    pub fn color_step(self, num: i32, steps: &[(i32, &'a str)]) -> Self {
        let mut color = None;
        for (lim, col) in steps.iter() {
            if num >= *lim {
                color = Some(col);
            } else {
                break;
            }
        }

        if let Some(col) = color {
            self.colorize(*col)
        } else {
            self
        }
    }

    pub fn add_trunc(self, max_len: usize, mut s: String) -> Self {
        if s.chars().count() > max_len {
            let stop = s.char_indices().into_iter().take(max_len).last().expect("max_len can't be 0").0;
            s.truncate(stop);
            self.add(s)
                .add("…")
        } else {
            self.add(s)
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

    let n = (num / 10_f64).trunc() as i32;
    let d = (num % 10_f64) as i32;

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

    let mut s = n.to_string();
    if d > 0 {
        s.push_str(".");
        s.push_str(&d.to_string());
    }
    s.push_str(" ");
    s.push_str(unit);

    s
}
