use x11rb::connection::Connection;
use x11rb::generated::randr::{get_screen_resources, get_output_info, get_crtc_info};
use x11rb::generated::xinerama::query_screens;
use std::collections::HashMap;

// type Rectangle = (i16, i16, u16, u16);

pub fn map_output_to_xinerama() -> Result<HashMap<String, usize>, Box<dyn std::error::Error>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let root = conn.setup().roots[screen_num].root;

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

    let xinerama_rect = query_screens(&conn)?.reply()?.screen_info.into_iter()
        .enumerate()
        .map(|(i, si)| (i+1, (si.x_org, si.y_org, si.width, si.height)))
        .collect::<Vec<_>>();

    let find_xinerama = |rect| xinerama_rect.iter()
        .find(|(_, rect2)| rect == *rect2).expect("this should find something").0;

    Ok(crtc_rect.into_iter()
       .map(|(name, rect)| (name, find_xinerama(rect)))
       .collect())
}
