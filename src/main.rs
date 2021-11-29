#![deny(warnings)]

#[macro_use]
extern crate clap;

use clap::{App, Arg};
use kmeans_colors::get_kmeans_hamerly;
use palette::{Lab, Pixel, Srgb, Srgba};

mod get_bytes;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let matches =
        App::new("dominant_colours")
            .version(VERSION)
            .author("Alex Chan <alex@alexwlchan.net>")
            .about("Find the dominant colours in an image")
            .arg(
                Arg::with_name("PATH")
                    .help("path to the image to inspect")
                    .required(true)
                    .index(1)
            )
            .arg(
                Arg::with_name("MAX-COLOURS")
                    .long("max-colours")
                    .help("how many colours to find")
                    .default_value("5")
                    .takes_value(true)
            )
            .arg(
                Arg::with_name("no-palette")
                    .long("no-palette")
                    .help("Just print the hex values, not colour previews")
                    .takes_value(false)
            )
            .get_matches();

    // This .unwrap() is safe because "path" is a required param
    let path = matches.value_of("PATH").unwrap();

    // Get the max colours as a number.
    // See https://github.com/clap-rs/clap/blob/v2.33.1/examples/12_typed_values.rs
    let max_colours = value_t!(matches, "MAX-COLOURS", usize).unwrap_or_else(|e| e.exit());

    let img_bytes = if path.ends_with(".gif") {
        get_bytes::get_bytes_for_gif(path)
    } else {
        get_bytes::get_bytes_for_image(path)
    };

    // This is based on code from the kmeans-colors binary, but with a bunch of
    // the options stripped out.
    // See https://github.com/okaneco/kmeans-colors/blob/9960c55dbc572e08d564dc341d6fd7e66fa79b5e/src/bin/kmeans_colors/app.rs
    let lab: Vec<Lab> = Srgba::from_raw_slice(&img_bytes)
        .iter()
        .map(|x| x.into_format().into())
        .collect();

    let max_iterations = 20;
    let converge = 1.0;
    let verbose = false;
    let seed: u64 = 0;

    let result = get_kmeans_hamerly(max_colours, max_iterations, converge, verbose, &lab, seed);

    let rgb = &result.centroids
        .iter()
        .map(|x| Srgb::from(*x).into_format())
        .collect::<Vec<Srgb<u8>>>();

    // This uses ANSI escape sequences and Unicode block elements to print
    // a palette of hex strings which are coloured to match.
    // See https://alexwlchan.net/2021/04/coloured-squares/
    for c in rgb {
        let display_value = format!("#{:02x}{:02x}{:02x}", c.red, c.green, c.blue);

        if matches.is_present("no-palette") {
            println!("{}", display_value);
        } else {
            println!("\x1B[38;2;{};{};{}m▇ {}\x1B[0m", c.red, c.green, c.blue, display_value);
        }
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
            output.stdout == "\u{1b}[38;2;255;0;0m▇ #ff0000\u{1b}[0m\n" ||
            output.stdout == "\u{1b}[38;2;254;0;0m▇ #fe0000\u{1b}[0m\n",
            "stdout = {:?}", output.stdout
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
    fn it_omits_the_escape_codes_with_no_palette() {
        let output = get_success(&["./src/tests/red.png", "--max-colours=1"]);

        assert_eq!(output.exit_code, 0);

        assert!(
            output.stdout == "\u{1b}[38;2;255;0;0m▇ #ff0000\u{1b}[0m\n" ||
            output.stdout == "\u{1b}[38;2;254;0;0m▇ #fe0000\u{1b}[0m\n",
            "stdout = {:?}", output.stdout
        );

        assert_eq!(output.stderr, "");
    }

    #[test]
    fn it_defaults_to_five_colours() {
        let output = get_success(&["./src/tests/noise.jpg"]);

        assert_eq!(output.stdout.matches("\n").count(), 5, "stdout = {:?}", output.stdout);
    }

    #[test]
    fn it_lets_you_choose_the_max_colours() {
        let output = get_success(&["./src/tests/noise.jpg", "--max-colours=8"]);

        assert_eq!(output.stdout.matches("\n").count(), 8, "stdout = {:?}", output.stdout);
    }

    #[test]
    fn it_looks_at_multiple_frames_in_an_animated_gif() {
        let output = get_success(&["./src/tests/animated_squares.gif"]);

        assert_eq!(output.stdout.matches("\n").count(), 2, "stdout = {:?}", output.stdout);
    }

    #[test]
    fn it_fails_if_you_pass_an_invalid_max_colours() {
        let output = get_failure(&["./src/tests/red.png", "--max-colours=NaN"]);

        assert_eq!(output.exit_code, 1);
        assert_eq!(output.stdout, "");
        assert_eq!(output.stderr, "error: Invalid value: The argument 'NaN' isn't a valid value\n");
    }

    #[test]
    fn it_fails_if_you_pass_an_nonexistent_file() {
        let output = get_failure(&["./doesnotexist.jpg"]);

        assert_eq!(output.exit_code, 1);
        assert_eq!(output.stdout, "");
        assert_eq!(output.stderr, "No such file or directory (os error 2)\n");
    }

    #[test]
    fn it_fails_if_you_pass_a_non_image_file() {
        let output = get_failure(&["./README.md"]);

        assert_eq!(output.exit_code, 1);
        assert_eq!(output.stdout, "");
        assert_eq!(output.stderr, "The file extension `.\"md\"` was not recognized as an image format\n");
    }

    #[test]
    fn it_fails_if_you_pass_an_unsupported_image_format() {
        let output = get_failure(&["./src/tests/green.tiff"]);

        assert_eq!(output.exit_code, 1);
        assert_eq!(output.stdout, "");
        assert_eq!(output.stderr, "The image format Tiff is not supported\n");
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
        let output = cmd
            .args(args)
            .unwrap_err()
            .as_output()
            .unwrap()
            .to_owned();

        DcOutput {
            exit_code: output.status.code().unwrap(),
            stdout: str::from_utf8(&output.stdout).unwrap().to_owned(),
            stderr: str::from_utf8(&output.stderr).unwrap().to_owned(),
        }
    }
}
