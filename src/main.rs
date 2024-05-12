#![deny(warnings)]

#[macro_use]
extern crate clap;

use palette::{Lab, Srgb};

mod cli;
mod find_dominant_colors;
mod get_image_colors;

fn main() {
    let matches = cli::app().get_matches();

    let path = matches
        .get_one::<String>("PATH")
        .expect("`path` is required");

    let max_colours: usize = *matches
        .get_one::<usize>("MAX_COLOURS")
        .expect("`max-colours` is required");

    let lab: Vec<Lab> = get_image_colors::get_image_colors(&path);

    let dominant_colors = find_dominant_colors::find_dominant_colors(&lab, max_colours);

    let background = matches.get_one::<Srgb<u8>>("BACKGROUND_HEX");

    // This uses ANSI escape sequences and Unicode block elements to print
    // a palette of hex strings which are coloured to match.
    // See https://alexwlchan.net/2021/04/coloured-squares/
    for c in dominant_colors {
        print_color(c, &background, matches.get_flag("no-palette"));
    }
}

// Print the colours to the terminal, using ANSI escape codes to
// apply formatting if desired.
//
// Useful reading: https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797?permalink_comment_id=3857871
fn print_color(c: Srgb<u8>, background: &Option<&Srgb<u8>>, no_palette: bool) {
    let display_value = format!("#{:02x}{:02x}{:02x}", c.red, c.green, c.blue);

    if no_palette {
        println!("{}", display_value);
    } else {
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
    }
}

#[cfg(test)]
mod tests {
    use std::str;

    use assert_cmd::assert::OutputAssertExt;
    use assert_cmd::Command;

    // Note: for the purposes of these tests, I mostly trust the k-means code
    // provided by the external library.

    #[test]
    fn it_prints_the_color_with_ansi_escape_codes() {
        let output = get_success(&["./src/tests/red.png", "--max-colours=1"]);

        assert_eq!(output.exit_code, 0);

        assert!(
            output.stdout == "\u{1b}[38;2;255;0;0m▇ #ff0000\u{1b}[0m\n"
                || output.stdout == "\u{1b}[38;2;254;0;0m▇ #fe0000\u{1b}[0m\n",
            "stdout = {:?}",
            output.stdout
        );

        assert_eq!(output.stderr, "");
    }

    #[test]
    fn it_can_look_at_png_images() {
        let output = get_success(&["./src/tests/red.png", "--max-colours=1"]);
        assert_eq!(output.exit_code, 0);
    }

    #[test]
    fn it_can_look_at_jpeg_images() {
        let output = get_success(&["./src/tests/noise.jpg", "--max-colours=1"]);
        assert_eq!(output.exit_code, 0);
    }

    #[test]
    fn it_can_look_at_static_gif_images() {
        let output = get_success(&["./src/tests/yellow.gif", "--max-colours=1"]);
        assert_eq!(output.exit_code, 0);
    }

    #[test]
    fn it_can_look_at_tiff_images() {
        let output = get_success(&["./src/tests/green.tiff", "--max-colours=1"]);
        assert_eq!(output.exit_code, 0);
    }

    #[test]
    fn it_omits_the_escape_codes_with_no_palette() {
        let output = get_success(&["./src/tests/red.png", "--max-colours=1", "--no-palette"]);

        assert_eq!(output.exit_code, 0);

        assert!(
            output.stdout == "#ff0000\n" || output.stdout == "#fe0000\n",
            "stdout = {:?}",
            output.stdout
        );

        assert_eq!(output.stderr, "");
    }

    #[test]
    fn it_defaults_to_five_colours() {
        let output = get_success(&["./src/tests/noise.jpg"]);

        assert_eq!(
            output.stdout.matches("\n").count(),
            5,
            "stdout = {:?}",
            output.stdout
        );
    }

    #[test]
    fn it_lets_you_choose_the_max_colours() {
        let output = get_success(&["./src/tests/noise.jpg", "--max-colours=8"]);

        assert_eq!(
            output.stdout.matches("\n").count(),
            8,
            "stdout = {:?}",
            output.stdout
        );
    }

    // The image created in the next two tests was created with the
    // following command:
    //
    //      convert -delay 200 -loop 10 -dispose previous red.png blue.png red.png blue.png red.png blue.png red.png blue.png animated_squares.gif
    //

    #[test]
    fn it_looks_at_multiple_frames_in_an_animated_gif() {
        let output = get_success(&["./src/tests/animated_squares.gif"]);

        assert_eq!(
            output.stdout.matches("\n").count(),
            2,
            "stdout = {:?}",
            output.stdout
        );
    }

    #[test]
    fn it_looks_at_multiple_frames_in_an_animated_gif_uppercase() {
        let output = get_success(&["./src/tests/animated_upper_squares.GIF"]);

        assert_eq!(
            output.stdout.matches("\n").count(),
            2,
            "stdout = {:?}",
            output.stdout
        );
    }

    #[test]
    fn it_fails_if_you_pass_an_invalid_max_colours() {
        let output = get_failure(&["./src/tests/red.png", "--max-colours=NaN"]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(output.stdout, "");
        assert_eq!(
            output.stderr,
            "error: invalid value 'NaN' for '--max-colours <MAX-COLOURS>': invalid digit found in string\n\nFor more information, try '--help'.\n"
        );
    }

    #[test]
    fn it_fails_if_you_pass_an_nonexistent_file() {
        let output = get_failure(&["./doesnotexist.jpg"]);

        assert_eq!(output.exit_code, 1);
        assert_eq!(output.stdout, "");
        assert_eq!(output.stderr, "No such file or directory (os error 2)\n");
    }

    #[test]
    fn it_fails_if_you_pass_an_nonexistent_gif() {
        let output = get_failure(&["./doesnotexist.gif"]);

        assert_eq!(output.exit_code, 1);
        assert_eq!(output.stdout, "");
        assert_eq!(output.stderr, "No such file or directory (os error 2)\n");
    }

    #[test]
    fn it_fails_if_you_pass_a_non_image_file() {
        let output = get_failure(&["./README.md"]);

        assert_eq!(output.exit_code, 1);
        assert_eq!(output.stdout, "");
        assert_eq!(output.stderr, "The image format could not be determined\n");
    }

    #[test]
    fn it_fails_if_you_pass_an_unsupported_image_format() {
        let output = get_failure(&["./src/tests/orange.heic"]);

        assert_eq!(output.exit_code, 1);
        assert_eq!(output.stdout, "");
        assert_eq!(output.stderr, "The image format could not be determined\n");
    }

    #[test]
    fn it_fails_if_you_pass_a_malformed_image() {
        let output = get_failure(&["./src/tests/malformed.txt.png"]);

        assert_eq!(output.exit_code, 1);
        assert_eq!(output.stdout, "");
        assert_eq!(
            output.stderr,
            "Format error decoding Png: Invalid PNG signature.\n"
        );
    }

    struct DcOutput {
        exit_code: i32,
        stdout: String,
        stderr: String,
    }

    fn get_success(args: &[&str]) -> DcOutput {
        let mut cmd = Command::cargo_bin("dominant_colours").unwrap();
        let output = cmd
            .args(args)
            .unwrap()
            .assert()
            .success()
            .get_output()
            .to_owned();

        DcOutput {
            exit_code: output.status.code().unwrap(),
            stdout: str::from_utf8(&output.stdout).unwrap().to_owned(),
            stderr: str::from_utf8(&output.stderr).unwrap().to_owned(),
        }
    }

    fn get_failure(args: &[&str]) -> DcOutput {
        let mut cmd = Command::cargo_bin("dominant_colours").unwrap();
        let output = cmd.args(args).unwrap_err().as_output().unwrap().to_owned();

        DcOutput {
            exit_code: output.status.code().unwrap(),
            stdout: str::from_utf8(&output.stdout).unwrap().to_owned(),
            stderr: str::from_utf8(&output.stderr).unwrap().to_owned(),
        }
    }
}
