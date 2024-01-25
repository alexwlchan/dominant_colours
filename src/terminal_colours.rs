use palette::Srgb;

const UNIX_COLORS: [[u8; 3]; 16] = [
    // Darker colors
    [0, 0, 0], // black
    [128, 0, 0], // dark red
    [0, 128, 0], // dark green
    [128, 128, 0], // dark yellow
    [0, 0, 128], // dark blue
    [128, 0, 128], // dark pink
    [0, 128, 128], // dark cyan
    [192, 192, 192], // light grey

    // Lighter colors
    [128, 128, 128], // grey
    [255, 0, 0], // red
    [0, 255, 0], // green
    [255, 255, 0], // yellow
    [0, 0, 255], // blue
    [255, 0, 255], // pink
    [0, 255, 255], // cyan
    [255, 255, 255], // white
];

pub fn map_to_terminal_color(colors: Vec<Srgb<u8>>) -> Vec<Srgb<u8>> {
    let colors : Vec<[u8; 3]> = colors.iter().map(|rgb| [rgb.red, rgb.green, rgb.blue]).collect();

    let mut result: Vec<Srgb<u8>> = Vec::new();
    for unix_color in UNIX_COLORS {
        let mut closest_color = [0,0,0];
        let mut smallest_distance = f64::MAX;

        for color in &colors {
            let distance = color_distance(unix_color, *color);
            if distance < smallest_distance {
                smallest_distance = distance;
                closest_color = *color;
            }
        }
        result.push(Srgb::new(closest_color[0], closest_color[1], closest_color[2]));
    }

    result
}

fn color_distance(c1: [u8; 3], c2: [u8; 3]) -> f64 {
    let r_mean = (c1[0] as f64 + c2[0] as f64) / 2.0;
    let r = c1[0] as f64 - c2[0] as f64;
    let g = c1[1] as f64 - c2[1] as f64;
    let b = c1[2] as f64 - c2[2] as f64;

    (((512.0+r_mean)*r*r)/256.0 + 4.0*g*g + ((767.0-r_mean)*b*b)/256.0).sqrt()
    // (r*r + g*g + b*b).sqrt()
}