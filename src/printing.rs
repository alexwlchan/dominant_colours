use palette::Srgb;

// Print the colours to the terminal, using ANSI escape codes to
// apply formatting if desired.
//
// See https://alexwlchan.net/2021/04/coloured-squares/
// See: https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797?permalink_comment_id=3857871
pub fn print_color(
    c: Srgb<u8>,
    background: &Option<Srgb<u8>>,
    include_bg_color: bool,
    uppecase_hex: bool,
) {
    let display_value = if uppecase_hex {
        format!("#{:02X}{:02X}{:02X}", c.red, c.green, c.blue)
    } else {
        format!("#{:02x}{:02x}{:02x}", c.red, c.green, c.blue)
    };

    if include_bg_color {
        // If a background colour is specified, print it behind the
        // hex strings.
        match background {
            Some(bg) => print!("\x1B[48;2;{};{};{}m", bg.red, bg.green, bg.blue),
            _ => (),
        };

        println!(
            "\x1B[38;2;{};{};{}m▇ {}\x1B[0m",
            c.red, c.green, c.blue, display_value
        );
    } else {
        println!("{}", display_value);
    }
}
