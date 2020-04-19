use std::borrow::Cow;

pub struct Parsed<'a> {
    tokens: Vec<Cow<'a, str>>
}

impl<'a> Parsed<'a> {
    pub fn parse(s: &'a str) -> Self {
        let tag = regex::Regex::new(r"(^|[^^])(\^[a-z]{1,2}\()").unwrap();
        let mut v = Vec::new();
        let mut i = 0;
        while let Some(cap) = tag.captures(&s[i..]) {
            let mat = cap.get(2).unwrap();
            if 0 < mat.start() {
                v.push(&s[i..i+mat.start()]);
            }
            v.push(&s[i+mat.start()..i+mat.end()]);
            if mat.end() < s[i..].len() {
                if let Some(p) = find_end_par(&s[i+mat.end()..]) {
                    v.push(&s[i+mat.end()..i+mat.end()+p]);
                    v.push(")");
                    let par_len = ')'.len_utf8();
                    i += mat.end()+p+par_len;
                } else {
                    i += mat.end();
                }
            }
        }

        if i < s.len() {
            v.push(&s[i..]);
        }

        Parsed {
            tokens: v.into_iter().map(|x| x.into()).collect()
        }
    }

    pub fn map_tag<F,T>(&mut self, f: F)
    where F: Fn(&str, &str) -> T,
          T: Into<Cow<'a, str>>
    {
        for i in 0..(self.tokens.len() - 2) {
            if self.tokens[i].as_ref().starts_with("^") {
                let n = f(&self.tokens[i], &self.tokens[i+1]);
                self.tokens[i+1] = n.into();
            }
        }
    }

    pub fn to_string(self) -> String {
        self.tokens.into_iter().collect()
    }
}

fn find_end_par(s: &str) -> Option<usize> {
    let mut p = 1;
    for (i, c) in s.char_indices() {
        match c {
            ')' => {
                p -= 1;
            }
            '(' => {
                p += 1;
            }
            _ => ()
        }
        if p == 0 {
            return Some(i);
        }
    }
    None
}
