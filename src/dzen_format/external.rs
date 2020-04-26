use crate::dzen_format::parser;
use std::borrow::Cow;

// prepend path to all icons and fix colors according to theme
pub fn fix_dzen_string<S>(s: S) -> String
where S: AsRef<str>
{
    let mut p = parser::Parsed::parse(s.as_ref());
    p.map_tag(|tag, cont| match tag {
        "i" => {
            let themed = crate::config::icon_theme(cont).unwrap_or(cont);
            let mut pathed = String::new() + crate::config::ICON_PATH + "/" + themed + ".xpm";
            if pathed.starts_with("~") {
                pathed.replace_range(..1, unsafe{&crate::HOME});
            }
            Cow::from(pathed)
        }
        "fg" | "bg" => crate::config::theme(cont).map_or_else(|| Cow::from(cont),
                                                              |s| Cow::from(s)),
        _ => Cow::from(cont)
    });
    p.to_string()
}
