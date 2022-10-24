// #![deny(warnings)]

#[macro_use]
extern crate clap;

use kmeans_colors::get_kmeans_hamerly;
use ordered_float::OrderedFloat;
use palette::{FromColor, Hsv, IntoColor, Lab, Pixel, RelativeContrast, Srgb, Srgba};
use std::str::FromStr;

mod cli;
mod get_bytes;
mod models;

fn main() {
    let matches = cli::app().get_matches();
    let action = cli::parse_arguments(matches);

    // There's different code for fetching bytes from GIF images because
    // GIFs are often animated, and we want a selection of frames.
    let img_bytes = if action.path.to_lowercase().ends_with(".gif") {
        get_bytes::get_bytes_for_gif(&action.path)
    } else {
        get_bytes::get_bytes_for_image(&action.path)
    };

    // This is based on code from the kmeans-colors binary, but with a bunch of
    // the options stripped out.
    // See https://github.com/okaneco/kmeans-colors/blob/0.5.0/src/bin/kmeans_colors/app.rs
    let lab: Vec<Lab> = Srgba::from_raw_slice(&img_bytes)
        .iter()
        .map(|x| x.into_format::<_, f32>().into_color())
        .collect();

    let dominant_colours = get_dominant_colours(lab, action.max_colours);

    let rgb: Vec<Srgb<u8>> = match action.options {
        models::ActionOptions::GetDominantColours => dominant_colours,
        models::ActionOptions::GetBestColourWith { compared_to } => {
            vec![find_best_colour_compared_to(dominant_colours, compared_to)]
        }
    };

    for c in rgb {
        print_hex_string(c, action.no_palette);
    }
}

/** Find the "best" colour from a set of candidates to use with
 * a given background colour.
 *
 * Here "best" is defined by two things:
 *
 *    - having at least a 4.5:1 contrast ratio, which passes WCAG AA
 *    - having the max possible saturation, to select for bright/fun colours
 *
 */
fn find_best_colour_compared_to(rgb: Vec<Srgb<u8>>, background_color: Srgb<u8>) -> Srgb<u8> {
    // Every colour in the RGB space has a contrast ratio of 4.5:1 with
    // at least one of black or white, so we can use one of these as an
    // extreme if we have to.
    //
    // Note: you could modify the dominant colours until one of them
    // has sufficient contrast, but that's omitted here because it adds
    // a lot of complexity for a relatively unusual case.
    let black_and_white: Vec<Srgb<u8>> = vec![
        Srgb::<u8>::from_str("#ffffff").unwrap(),
        Srgb::<u8>::from_str("#000000").unwrap(),
    ];
    let candidates = [rgb, black_and_white].concat();

    // Now filter out all the colours that have a contrast ratio with
    // the background which is less than 4.5:1.
    let high_contrast_candidates: Vec<Srgb<u8>> = candidates
        .into_iter()
        .filter(|c| {
            RelativeContrast::get_contrast_ratio(
                &c.into_format::<f32>(),
                &background_color.into_format::<f32>(),
            ) >= 4.5
        })
        .collect();

    // And now let's maximise for saturation.  We know the final unwrap()
    // is safe because `high_contrast_candidates` will always be non-empty --
    // that's from adding black and white earlier.
    assert!(
        high_contrast_candidates.len() > 0,
        "found no colours with sufficient contrast"
    );
    high_contrast_candidates
        .into_iter()
        .max_by_key(|rgb| OrderedFloat(Hsv::from_color(rgb.into_format::<f32>()).saturation))
        .unwrap()
}

fn get_dominant_colours(lab: Vec<Lab>, max_colours: usize) -> Vec<Srgb<u8>> {
    let max_iterations = 20;
    let converge = 1.0;
    let verbose = false;
    let seed: u64 = 0;

    let result = get_kmeans_hamerly(max_colours, max_iterations, converge, verbose, &lab, seed);

    let rgb: Vec<Srgb<u8>> = result
        .centroids
        .iter()
        .map(|x| Srgb::from_color(*x).into_format())
        .collect::<Vec<Srgb<u8>>>();

    rgb
}

fn print_hex_string(c: Srgb<u8>, no_palette: bool) -> () {
    // This uses ANSI escape sequences and Unicode block elements to print
    // a palette of hex strings which are coloured to match.
    // See https://alexwlchan.net/2021/04/coloured-squares/
    let display_value = format!("#{:02x}{:02x}{:02x}", c.red, c.green, c.blue);

    if no_palette {
        println!("{}", display_value);
    } else {
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
    fn it_finds_the_best_colour_against_a_background() {
        // This is an image with a grey stripe and a green stripe; both
        // have sufficient contrast with a black background, so we check
        // it picks the more poppy green.
        let output = get_success(&[
            "./src/tests/split-grey-green.png",
            "--no-palette",
            "--compared-to",
            "#000000",
        ]);

        assert_eq!(output.stdout, "#8efa00\n");
    }

    #[test]
    fn it_picks_black_if_no_colour_is_dark_enough() {
        // This is an image with two light grey stripes, neither of which
        // have enough contrast -- so we should pick black instead.
        let output = get_success(&[
            "./src/tests/split-grey-grey-light.png",
            "--no-palette",
            "--compared-to",
            "#ffffff",
        ]);

        assert_eq!(output.stdout, "#000000\n");
    }

    #[test]
    fn it_picks_white_if_no_colour_is_light_enough() {
        // This is an image with two dark grey stripes, neither of which
        // have enough contrast -- so we should pick white instead.
        let output = get_success(&[
            "./src/tests/split-grey-grey-dark.png",
            "--no-palette",
            "--compared-to",
            "#000000",
        ]);

        assert_eq!(output.stdout, "#ffffff\n");
    }

    #[test]
    fn it_fails_if_you_pass_an_invalid_max_colours() {
        let output = get_failure(&["./src/tests/red.png", "--max-colours=NaN"]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(output.stdout, "");
        assert_eq!(
            output.stderr,
            "error: Invalid value 'NaN' for '--max-colours <MAX-COLOURS>': invalid digit found in string\n\nFor more information try '--help'\n"
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
        assert_eq!(
            output.stderr,
            "The file extension `.\"md\"` was not recognized as an image format\n"
        );
    }

    #[test]
    fn it_fails_if_you_pass_an_unsupported_image_format() {
        let output = get_failure(&["./src/tests/purple.webp"]);

        assert_eq!(output.exit_code, 1);
        assert_eq!(output.stdout, "");
        assert_eq!(output.stderr, "The image format WebP is not supported\n");
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
