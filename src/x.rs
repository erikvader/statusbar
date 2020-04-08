use x11rb::connection::Connection;
use x11rb::generated::randr::{get_screen_resources, get_output_info, get_crtc_info};
use x11rb::generated::xinerama::query_screens;

pub type Rectangle = (i16, i16, u16, u16);
pub struct XSetup {
    outputs: Vec<(String, usize, Rectangle)>
}

impl XSetup {
    #![allow(dead_code)]
    pub fn get_xinerama(&self, name: &str) -> Option<usize> {
        self.outputs.iter().find(|(s, _, _)| s == name).map(|(_, u, _)| *u)
    }

    pub fn get_name(&self, xinerama: usize) -> Option<&str> {
        self.outputs.iter().find(|(_, u, _)| *u == xinerama).map(|(s, _, _)| s.as_str())
    }

    pub fn get_rect(&self, name: &str) -> Option<Rectangle> {
        self.outputs.iter().find(|(s, _, _)| s == name).map(|(_, _, r)| *r)
    }

    pub fn outputs(&self) -> impl Iterator<Item = &str> {
        self.outputs.iter().map(|(o, _, _)| o.as_str())
    }
}

pub fn get_x_setup() -> Result<XSetup, Box<dyn std::error::Error>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let root = conn.setup().roots[screen_num].root;

    // query X for (output_name, rect)
    let mut crtc_rect = Vec::new();
    let res = get_screen_resources(&conn, root)?.reply()?;
    for o in res.outputs {
        let out_info = get_output_info(&conn, o, res.config_timestamp)?.reply()?;
        let crtc = out_info.crtc;
        if !out_info.crtcs.contains(&crtc) {
            continue;
        }
        let name = String::from_utf8(out_info.name)?;
        let info = get_crtc_info(&conn, crtc, res.config_timestamp)?.reply()?;
        crtc_rect.push((name, (info.x, info.y, info.width, info.height)));
    }

    // query xinerama for (xinerama_index, rect)
    let xinerama_rect = query_screens(&conn)?.reply()?.screen_info.into_iter()
        .enumerate()
        .map(|(i, si)| (i+1, (si.x_org, si.y_org, si.width, si.height)))
        .collect::<Vec<_>>();

    // combine both to (output_name, xinerama_index, rect)
    let find_xinerama = |rect| xinerama_rect.iter()
        .find(|(_, rect2)| rect == *rect2).expect("this should find something").0;
    let outputs = crtc_rect.into_iter()
        .map(|(name, rect)| (name, find_xinerama(rect), rect))
        .collect();

    Ok(XSetup{outputs})
}
