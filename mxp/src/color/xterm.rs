use std::sync::OnceLock;

use super::hex_color::HexColor;

pub fn xterm(code: u8) -> HexColor {
    xterm_colors()[code as usize]
}

pub fn xterm_colors() -> &'static [HexColor; 256] {
    static XTERM_COLORS: OnceLock<[HexColor; 256]> = OnceLock::new();
    XTERM_COLORS.get_or_init(create_xterm_colors)
}

const fn create_xterm_colors() -> [HexColor; 256] {
    let mut colors = [HexColor { code: 0 }; 256];
    colors[1] = HexColor::rgb(128, 0, 0); // maroon
    colors[2] = HexColor::rgb(0, 128, 0); // green
    colors[3] = HexColor::rgb(128, 128, 0); // olive
    colors[4] = HexColor::rgb(0, 0, 128); // navy
    colors[5] = HexColor::rgb(128, 0, 128); // purple
    colors[6] = HexColor::rgb(0, 128, 128); // teal
    colors[7] = HexColor::rgb(192, 192, 192); // silver
    colors[8] = HexColor::rgb(128, 128, 128); // gray
    colors[9] = HexColor::rgb(255, 0, 0); // red
    colors[10] = HexColor::rgb(0, 255, 0); // lime
    colors[11] = HexColor::rgb(255, 255, 0); // yellow
    colors[12] = HexColor::rgb(0, 0, 255); // blue
    colors[13] = HexColor::rgb(255, 0, 255); // magenta
    colors[14] = HexColor::rgb(0, 255, 255); // cyan
    colors[15] = HexColor::rgb(255, 255, 255); // white
    const COLOR_SCALE: &[u8] = &[
        0,
        95,
        95 + 40,
        95 + 40 + 40,
        95 + 40 + 40 + 40,
        95 + 40 + 40 + 40 + 40,
    ];
    let color_scale_len = COLOR_SCALE.len();
    let mut i = 16;

    let mut red_i = 0;
    while red_i < color_scale_len {
        let r = COLOR_SCALE[red_i];
        let mut green_i = 0;
        while green_i < color_scale_len {
            let g = COLOR_SCALE[green_i];
            let mut blue_i = 0;
            while blue_i < color_scale_len {
                let b = COLOR_SCALE[blue_i];
                colors[i] = HexColor::rgb(r, g, b);
                blue_i += 1;
            }
            green_i += 1;
        }
        red_i += 1;
    }
    let mut gray = 8;
    while gray < 248 {
        i += 1;
        colors[i] = HexColor::rgb(gray, gray, gray);
        gray += 10;
    }
    colors
}
