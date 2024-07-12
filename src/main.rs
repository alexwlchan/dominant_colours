#![deny(warnings)]

use std::path::PathBuf;

use clap::Parser;
use palette::{FromColor, Lab, Srgb};

mod find_dominant_colors;
mod get_image_colors;
mod printing;

#[derive(Parser, Debug)]
#[command(version, about = "Find the dominant colours in an image", long_about=None)]
struct Cli {
    /// Path to the image to inspect
    path: PathBuf,

    /// How many colours to find
    #[arg(long = "max-colours", default_value_t = 5)]
    max_colours: usize,

    /// Find a single colour that will look best against this background
    #[arg(long = "best-against-bg")]
    background: Option<Srgb<u8>>,

    /// Just print the hex values, not colour previews
    #[arg(long = "no-palette")]
    no_palette: bool,
}

fn main() {
    let cli = Cli::parse();

    let lab: Vec<Lab> = get_image_colors::get_image_colors(&cli.path);

    let dominant_colors = find_dominant_colors::find_dominant_colors(&lab, cli.max_colours);

    let selected_colors = match cli.background {
        Some(bg) => find_dominant_colors::choose_best_color_for_bg(dominant_colors.clone(), &bg),
        None => dominant_colors,
    };

    let rgb_colors = selected_colors
        .iter()
        .map(|c| Srgb::from_color(*c).into_format())
        .collect::<Vec<Srgb<u8>>>();

    for c in rgb_colors {
        printing::print_color(c, &cli.background, cli.no_palette);
    }
}

#[cfg(test)]
mod tests {
    use std::str;

    use assert_cmd::assert::OutputAssertExt;
    use assert_cmd::Command;
    use regex::Regex;

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
            "error: invalid value 'NaN' for '--max-colours <MAX_COLOURS>': invalid digit found in string\n\nFor more information, try '--help'.\n"
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

    #[test]
    fn it_chooses_the_right_color_for_a_dark_background() {
        let output = get_success(&[
            "src/tests/stripes.png",
            "--max-colours=5",
            "--best-against-bg=#222",
            "--no-palette",
        ]);

        assert_eq!(output.stdout, "#d4fb79\n");
    }

    #[test]
    fn it_chooses_the_right_color_for_a_light_background() {
        let output = get_success(&[
            "src/tests/stripes.png",
            "--max-colours=5",
            "--best-against-bg=#fff",
            "--no-palette",
        ]);

        assert_eq!(output.stdout, "#693900\n");
    }

    #[test]
    fn it_prints_the_version() {
        let output = get_success(&["--version"]);

        let re = Regex::new(r"^dominant_colours [0-9]+\.[0-9]+\.[0-9]+\n$").unwrap();

        assert!(re.is_match(&output.stdout));

        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stderr, "");
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
