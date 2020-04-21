use crate::dzen_format::parser;
use std::borrow::Cow;

// prepend path to all icons and fix colors according to theme
pub fn fix_dzen_string<S>(s: S) -> String
where S: AsRef<str>
{
    let mut p = parser::Parsed::parse(s.as_ref());
    p.map_tag(|tag, cont| match tag {
        "i" => {
            let mut pathed = String::new() + crate::config::ICON_PATH + "/" + cont;
            if pathed.starts_with("~") {
                pathed.replace_range(..1, unsafe{&crate::HOME});
            }
            Cow::from(pathed)
        }
        "fg" | "bg" => Cow::from(crate::config::theme(cont)),
        _ => Cow::from(cont)
    });
    p.to_string()
}
