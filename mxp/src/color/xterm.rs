use super::rgb::RgbColor;

impl RgbColor {
    /// Standard definitions for 3-bit color.
    pub const XTERM_8: &'static [Self; 8] = first_xterm_colors();

    /// Standard definitions for 4-bit color.
    pub const XTERM_16: &'static [Self; 16] = first_xterm_colors();

    /// Standard definitions for 8-bit color.
    pub const XTERM_256: &'static [Self; 256] = &create_xterm_colors();

    /// Translates an 8-bit integer into an 8-bit color.
    pub const fn xterm(code: u8) -> Self {
        RgbColor::XTERM_256[code as usize]
    }
}

// Will be unnecessary once const Option::unwrap is stabilized.
const fn first_xterm_colors<const N: usize>() -> &'static [RgbColor; N] {
    match RgbColor::XTERM_256.first_chunk() {
        Some(chunk) => chunk,
        None => unreachable!(),
    }
}

const fn create_xterm_colors() -> [RgbColor; 256] {
    const COLOR_SCALE: &[u8] = &[
        0,
        95,
        95 + 40,
        95 + 40 + 40,
        95 + 40 + 40 + 40,
        95 + 40 + 40 + 40 + 40,
    ];
    const COLOR_SCALE_LEN: usize = COLOR_SCALE.len();

    let mut colors = [RgbColor::rgb(0, 0, 0); 256];
    colors[1] = RgbColor::rgb(128, 0, 0); // maroon
    colors[2] = RgbColor::rgb(0, 128, 0); // green
    colors[3] = RgbColor::rgb(128, 128, 0); // olive
    colors[4] = RgbColor::rgb(0, 0, 128); // navy
    colors[5] = RgbColor::rgb(128, 0, 128); // purple
    colors[6] = RgbColor::rgb(0, 128, 128); // teal
    colors[7] = RgbColor::rgb(192, 192, 192); // silver
    colors[8] = RgbColor::rgb(128, 128, 128); // gray
    colors[9] = RgbColor::rgb(255, 0, 0); // red
    colors[10] = RgbColor::rgb(0, 255, 0); // lime
    colors[11] = RgbColor::rgb(255, 255, 0); // yellow
    colors[12] = RgbColor::rgb(0, 0, 255); // blue
    colors[13] = RgbColor::rgb(255, 0, 255); // magenta
    colors[14] = RgbColor::rgb(0, 255, 255); // cyan
    colors[15] = RgbColor::rgb(255, 255, 255); // white
    let mut i = 16;
    let mut red_i = 0;
    while red_i < COLOR_SCALE_LEN {
        let r = COLOR_SCALE[red_i];
        let mut green_i = 0;
        while green_i < COLOR_SCALE_LEN {
            let g = COLOR_SCALE[green_i];
            let mut blue_i = 0;
            while blue_i < COLOR_SCALE_LEN {
                let b = COLOR_SCALE[blue_i];
                colors[i] = RgbColor::rgb(r, g, b);
                i += 1;
                blue_i += 1;
            }
            green_i += 1;
        }
        red_i += 1;
    }
    let mut gray = 8;
    while i < 256 {
        colors[i] = RgbColor::rgb(gray, gray, gray);
        gray += 10;
        i += 1;
    }
    colors
}
