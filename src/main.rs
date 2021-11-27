#![deny(warnings)]

#[macro_use]
extern crate clap;

use clap::{App, Arg};
use image::imageops::FilterType;
use palette::{Lab, Pixel, Srgb, Srgba};
use kmeans_colors::{get_kmeans_hamerly};

fn main() {
    let matches =
        App::new("dominant_colours")
            .version("1.0")
            .author("Alex Chan <alex@alexwlchan.net>")
            .about("Find the dominant colours in an image")
            .arg(
                Arg::with_name("path")
                    .help("path to the image to inspect")
                    .required(true)
                    .index(1)
            )
            .arg(
                Arg::with_name("count")
                    .long("count")
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
    let path = matches.value_of("path").unwrap();

    // Get the count as a number.
    // See https://github.com/clap-rs/clap/blob/v2.33.1/examples/12_typed_values.rs
    let count = value_t!(matches, "count", usize).unwrap_or_else(|e| e.exit());

    // Open the image, then resize it.  For this tool I'd rather get a good answer
    // quickly than a great answer slower.
    //
    // The choice of max dimension is arbitrary.  Making it smaller means you get
    // faster results, but possibly at the loss of quality.
    //
    // The nearest neighbour algorithm produces images that don't look as good,
    // but it's much much faster and the loss of quality is unlikely to be
    // an issue when looking for dominant colours.
    //
    // Note: when trying to work out what's "fast enough", make sure you use release
    // mode.  The image/k-means operations are significantly faster (=2 orders
    // of magnitude) than in debug mode.
    //
    // See https://docs.rs/image/0.23.14/image/imageops/enum.FilterType.html
    let img = image::open(&path).unwrap();
    let resized_img = img.resize(400, 400, FilterType::Nearest);

    let img_vec = resized_img.into_rgba8().into_raw();

    // This is based on code from the kmeans-colors binary, but with a bunch of
    // the options stripped out.
    // See https://github.com/okaneco/kmeans-colors/blob/9960c55dbc572e08d564dc341d6fd7e66fa79b5e/src/bin/kmeans_colors/app.rs
    let lab: Vec<Lab> = Srgba::from_raw_slice(&img_vec)
        .iter()
        .map(|x| x.into_format().into())
        .collect();

    let max_iterations = 20;
    let converge = 1.0;
    let verbose = false;
    let seed: u64 = 0;

    let result = get_kmeans_hamerly(count, max_iterations, converge, verbose, &lab, seed);

    let rgb = &result.centroids
        .iter()
        .map(|x| Srgb::from(*x).into_format())
        .collect::<Vec<Srgb<u8>>>();

    // This uses ANSI escape sequences and Unicode block elements to print
    // a palette of hex strings which are coloured to match.
    // See https://alexwlchan.net/2021/04/coloured-squares/
    for c in rgb {
        if matches.is_present("no-palette") {
            println!("#{:02x}{:02x}{:02x}", c.red, c.green, c.blue);
        } else {
            println!("\x1B[38;2;{};{};{}m▇ #{:02x}{:02x}{:02x}\x1B[0m", c.red, c.green, c.blue, c.red, c.green, c.blue);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str;
    use std::process::Output;

    use assert_cmd::assert::OutputAssertExt;
    use assert_cmd::Command;

    // Note: for the purposes of these tests, I mostly trust the k-means code
    // provided by the external library.

    #[test]
    fn it_prints_the_color_with_ansi_escape_codes() {
        let output = get_success(&["./src/tests/red.png", "--count=1"]);

        assert_eq!(output.status.code().unwrap(), 0);

        let stdout = str::from_utf8(&output.stdout).unwrap();
        assert!(
            stdout == "\u{1b}[38;2;255;0;0m▇ #ff0000\u{1b}[0m\n" ||
            stdout == "\u{1b}[38;2;254;0;0m▇ #fe0000\u{1b}[0m\n",
            "stdout = {:?}", stdout
        );

        assert_eq!(str::from_utf8(&output.stderr).unwrap(), "");
    }

    #[test]
    fn it_omits_the_escape_codes_with_no_palette() {
        let output = get_success(&["./src/tests/red.png", "--count=1"]);

        assert_eq!(output.status.code().unwrap(), 0);

        let stdout = str::from_utf8(&output.stdout).unwrap();
        assert!(
            stdout == "\u{1b}[38;2;255;0;0m▇ #ff0000\u{1b}[0m\n" ||
            stdout == "\u{1b}[38;2;254;0;0m▇ #fe0000\u{1b}[0m\n",
            "stdout = {:?}", stdout
        );

        assert_eq!(str::from_utf8(&output.stderr).unwrap(), "");
    }

    #[test]
    fn it_defaults_to_five_colours() {
        let output = get_success(&["./src/tests/noise.jpg"]);

        let stdout = str::from_utf8(&output.stdout).unwrap();
        assert_eq!(stdout.matches("\n").count(), 5, "stdout = {:?}", stdout);
    }

    #[test]
    fn it_lets_you_choose_the_count() {
        let output = get_success(&["./src/tests/noise.jpg", "--count=8"]);

        let stdout = str::from_utf8(&output.stdout).unwrap();
        assert_eq!(stdout.matches("\n").count(), 8, "stdout = {:?}", stdout);
    }

    #[test]
    fn it_fails_if_you_pass_an_invalid_count() {
        let output = get_failure(&["./src/tests/red.png", "--count=NaN"]);

        assert_eq!(output.exit_code, 1);
        assert_eq!(output.stdout, "");
        assert_eq!(output.stderr, "error: Invalid value: The argument 'NaN' isn't a valid value\n");
    }

    struct DcOutput {
        exit_code: i32,
        stdout: &str,
        stderr: &str,
    }

    fn get_success(args: &[&str]) -> Output {
        let mut cmd = Command::cargo_bin("dominant_colours").unwrap();
        cmd
            .args(args)
            .unwrap()
            .assert()
            .success()
            .get_output()
            .to_owned()
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
            stdout: str::from_utf8(&output.stdout).unwrap(),
            stderr: str::from_utf8(&output.stderr).unwrap(),
        }
    }
}
