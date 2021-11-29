// #![deny(warnings)]

#[macro_use]
extern crate clap;

use std::fs::File;

use clap::{App, Arg};
use image::imageops::FilterType;
use palette::{Lab, Pixel, Srgb, Srgba};
use kmeans_colors::{get_kmeans_hamerly};
use image::codecs::gif::GifDecoder;
use image::AnimationDecoder;
use image::{Frames, ImageBuffer, DynamicImage};

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

    let decoder = GifDecoder::new(File::open(&path).unwrap()).ok().unwrap();
    let frames2: Frames = decoder.into_frames();
    let frames3 = frames2.collect_frames().unwrap();

    let increment = if frames3.len() <= 50 {
        1
    } else {
        let s: f32 = (frames3.len() as f32) / (25 as f32);
        s.floor() as i32
    };

    let buffers: Vec<Vec<u8>> = frames3.iter()
    .enumerate()
    .filter(|(i, _)| {
        (*i as f32 / increment as f32).floor() == (*i as f32 / increment as f32)
    }

    )
    .map(|(i, frame)| DynamicImage::ImageRgba8(frame.buffer().clone()).resize(100, 100, FilterType::Nearest).into_rgba8().into_raw())
    .collect();

    println!("incrmenet = {:?}", increment);
    println!("frame count = {:?}", frames3.len());
    println!("images count = {:?}", buffers.len());

    // let buffers: Vec<Vec<u8>> = images.iter().map(|img| img).collect();

    // std::process::exit(1);
    //
    // let mut frames = GifDecoder::new(File::open(&path).unwrap()).ok().unwrap()
    //     .into_frames().collect_frames().unwrap();
    // frames.truncate(25);
    //
    // println!("frame count = {:?}", frames.len());

    // let buffers: Vec<Vec<u8>> = frames.iter().map(|f| f.to_owned().into_buffer().into_raw()).collect();
    let bytes: Vec<u8> = buffers.into_iter().flatten().collect();

    println!("pixels = {:?}", bytes.len());

    let img = match image::open(&path) {
        Ok(im) => im,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        },
    };

    // Resize the image after we open it.  For this tool I'd rather get a good answer
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
    // println!("{:?}", img);
    let resized_img = img.resize(400, 400, FilterType::Nearest);
    println!("{}", path);

    let img_vec: Vec<u8> = resized_img.into_rgba8().into_raw();

    // This is based on code from the kmeans-colors binary, but with a bunch of
    // the options stripped out.
    // See https://github.com/okaneco/kmeans-colors/blob/9960c55dbc572e08d564dc341d6fd7e66fa79b5e/src/bin/kmeans_colors/app.rs
    let lab: Vec<Lab> = Srgba::from_raw_slice(&bytes)
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
