use palette::Srgb;

// https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
const ANSI_COLOUR_TABLE: [[u8; 3]; 16] = [
    // Darker colors
    [0, 0, 0], // Black
    [170, 0, 0], // Red
    [0, 170, 0], // Green
    [128, 128, 0], // Yellow
    [0, 0, 170], // Blue
    [170, 0, 170], // Magenta
    [0, 170, 170], // Cyan
    [170, 170, 170], // White

    // Lighter colors
    [85, 85, 85], // Bright Black (Gray)
    [255, 85, 85], // Bright Red
    [85, 255, 85], // Bright Green
    [255, 255, 85], // Bright Yellow
    [85, 85, 255], // Bright Blue
    [255, 85, 255], // Bright Magenta
    [85, 255, 255], // Bright Cyan
    [255, 255, 255], // Bright White
];

// Takes a vector of colors in the Srgb<u8> format and returns a vector of colors
// in the same format, but mapped to the closest color in the ANSI color table.
pub fn create_terminal_color(colors: Vec<Srgb<u8>>) -> Vec<Srgb<u8>> {
    let colors : Vec<[u8; 3]> = colors.iter().map(|rgb| [rgb.red, rgb.green, rgb.blue]).collect();

    let mut result: Vec<Srgb<u8>> = Vec::new();
    for unix_color in ANSI_COLOUR_TABLE {

        // Find the color with the closest distance to the ANSI colour table
        let mut smallest_distance = f64::MAX;
        let mut closest_color = [0,0,0];
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

// This function calculates the distance between two colors in the RGB color space.
// It uses a formula that takes into account the human perception of color differences.
fn color_distance(c1: [u8; 3], c2: [u8; 3]) -> f64 {
    let r = c1[0] as f64 - c2[0] as f64;
    let g = c1[1] as f64 - c2[1] as f64;
    let b = c1[2] as f64 - c2[2] as f64;

    // Apparently more pleasing to the human eyes
    // https://stackoverflow.com/a/9085524/6802309
    let r_mean = (c1[0] as f64 + c2[0] as f64) / 2.0;
    (((512.0+r_mean)*r*r)/256.0 + 4.0*g*g + ((767.0-r_mean)*b*b)/256.0).sqrt()

    // Alternatively: return (r*r + g*g + b*b).sqrt();
}