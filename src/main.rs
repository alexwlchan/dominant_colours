#![deny(warnings)]

use std::io::IsTerminal;
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

    let lab: Vec<Lab> = match get_image_colors::get_image_colors(&cli.path) {
        Ok(lab) => lab,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    assert!(lab.len() > 0);

    let dominant_colors = find_dominant_colors::find_dominant_colors(&lab, cli.max_colours);

    let selected_colors = match cli.background {
        Some(bg) => find_dominant_colors::choose_best_color_for_bg(dominant_colors.clone(), &bg),
        None => dominant_colors,
    };

    let rgb_colors = selected_colors
        .iter()
        .map(|c| Srgb::from_color(*c).into_format())
        .collect::<Vec<Srgb<u8>>>();

    // Should we print with colours in the terminal, or just sent text?
    //
    // When I created this tool, I had a `--no-palette` flag to suppress the
    // terminal colours, but I've since realised that I can look for the
    // presence of a TTY and disable colours if we're not in a terminal,
    // even if the user hasn't passed `--no-palette`.
    //
    // I'm keeping the old flag for backwards compatibility, but I might
    // retire it in a future v2 update.
    //
    // Note: because of the difficulty of simulating a TTY in automated tests,
    // this isn't tested properly -- but I'll notice quickly if this breaks!
    let include_bg_color = if cli.no_palette {
        false
    } else if std::io::stdout().is_terminal() {
        true
    } else {
        false
    };

    for c in rgb_colors {
        printing::print_color(c, &cli.background, include_bg_color);
    }
}

#[cfg(test)]
mod tests {
    use predicates::prelude::*;

    use crate::run_command;

    // Note: for the purposes of these tests, I mostly trust the k-means code
    // provided by the external library.

    #[test]
    fn it_prints_the_colour() {
        let result = run_command!("./src/tests/red.png");

        result.success().stdout("#fe0000\n").stderr("");
    }

    #[test]
    fn it_can_look_at_png_images() {
        let result = run_command!("./src/tests/red.png");

        result.success().stdout("#fe0000\n").stderr("");
    }

    #[test]
    fn it_can_look_at_jpeg_images() {
        let result = run_command!("./src/tests/black.jpg");

        result.success().stdout("#000000\n").stderr("");
    }

    #[test]
    fn it_can_look_at_static_gif_images() {
        let result = run_command!("./src/tests/yellow.gif");

        result.success().stdout("#fffb00\n").stderr("");
    }

    #[test]
    fn it_can_look_at_tiff_images() {
        let result = run_command!("./src/tests/green.tiff");

        result.success().stdout("#04ff02\n").stderr("");
    }

    #[test]
    fn it_omits_the_escape_codes_with_no_palette() {
        let result = run_command!("./src/tests/red.png", "--max-colours=1");

        result.success().stdout("#fe0000\n").stderr("");
    }

    #[test]
    fn it_defaults_to_five_colours() {
        let result = run_command!("./src/tests/noise.jpg");

        let has_five_lines = predicate::str::is_match(r"^(#[a-f0-9]{6}\n){5}$").unwrap();

        result.success().stdout(has_five_lines).stderr("");
    }

    #[test]
    fn it_lets_you_choose_the_max_colours() {
        let result = run_command!("./src/tests/noise.jpg", "--max-colours=8");

        let has_eight_lines = predicate::str::is_match(r"^(#[a-f0-9]{6}\n){8}$").unwrap();

        result.success().stdout(has_eight_lines).stderr("");
    }

    // The image created in the next two tests was created with the
    // following command:
    //
    //      convert \
    //        -delay 200 \
    //        -loop 10 \
    //        -dispose previous \
    //        red.png blue.png \
    //        red.png blue.png \
    //        red.png blue.png \
    //        red.png blue.png \
    //        animated_squares.gif
    //
    // It creates an animated GIF that has alternating red/blue frames.

    #[test]
    fn it_looks_at_multiple_frames_in_an_animated_gif() {
        let result = run_command!("./src/tests/animated_squares.gif");

        result.success().stdout("#0200ff\n#ff0000\n").stderr("");
    }

    #[test]
    fn it_looks_at_multiple_frames_in_an_animated_gif_uppercase() {
        let result = run_command!("./src/tests/animated_upper_squares.GIF");

        result.success().stdout("#0200ff\n#ff0000\n").stderr("");
    }

    // This is an animated WebP that has alternating red/blue frames.
    //
    // It needs to look at multiple frames to see both colours.
    #[test]
    fn it_looks_at_multiple_frames_in_an_animated_webp() {
        let result = run_command!("./src/tests/animated_squares.webp");

        result
            .success()
            .stdout("#0200ff\n#ff0100\n#ff0002\n")
            .stderr("");
    }

    #[test]
    fn it_fails_if_you_pass_an_invalid_max_colours() {
        let result = run_command!("./src/tests/red.png", "--max-colours=NaN");

        result
            .failure()
            .code(2)
            .stdout("")
            .stderr("error: invalid value 'NaN' for '--max-colours <MAX_COLOURS>': invalid digit found in string\n\nFor more information, try '--help'.\n");
    }

    #[test]
    fn it_fails_if_you_pass_an_nonexistent_file() {
        let result = run_command!("./doesnotexist.jpg");

        result
            .failure()
            .code(1)
            .stdout("")
            .stderr("No such file or directory (os error 2)\n");
    }

    #[test]
    fn it_fails_if_you_pass_an_nonexistent_gif() {
        let result = run_command!("./doesnotexist.gif");

        result
            .failure()
            .code(1)
            .stdout("")
            .stderr("No such file or directory (os error 2)\n");
    }

    #[test]
    fn it_fails_if_you_pass_a_non_image_file() {
        let result = run_command!("./README.md");

        result
            .failure()
            .code(1)
            .stdout("")
            .stderr("Unable to determine image format from file extension\n");
    }

    #[test]
    fn it_fails_if_you_pass_an_unsupported_image_format() {
        let result = run_command!("./src/tests/orange.heic");

        result
            .failure()
            .code(1)
            .stdout("")
            .stderr("Unable to determine image format from file extension\n");
    }

    #[test]
    fn it_fails_if_you_pass_a_malformed_image() {
        let result = run_command!("./src/tests/malformed.txt.png");

        result
            .failure()
            .code(1)
            .stdout("")
            .stderr("Format error decoding Png: Invalid PNG signature.\n");
    }

    #[test]
    fn it_fails_if_you_pass_a_malformed_gif() {
        let result = run_command!("./src/tests/malformed.txt.gif");

        result
            .failure()
            .code(1)
            .stdout("")
            .stderr("Format error decoding Gif: malformed GIF header\n");
    }

    #[test]
    fn it_fails_if_you_pass_a_malformed_webp() {
        let result = run_command!("./src/tests/malformed.txt.webp");

        result
            .failure()
            .code(1)
            .stdout("")
            .stderr("Format error decoding WebP: Invalid Chunk header: [52, 49, 46, 46]\n");
    }

    #[test]
    fn it_fails_if_you_pass_a_path_without_a_file_extension() {
        let result = run_command!("./src/tests/noextension");

        result
            .failure()
            .code(1)
            .stdout("")
            .stderr("Path has no file extension, so could not determine image format\n");
    }

    #[test]
    fn it_chooses_the_right_color_for_a_dark_background() {
        let result = run_command!(
            "src/tests/stripes.png",
            "--max-colours=5",
            "--best-against-bg=#222",
        );

        result.success().stdout("#d4fb79\n").stderr("");
    }

    #[test]
    fn it_chooses_the_right_color_for_a_light_background() {
        let result = run_command!(
            "src/tests/stripes.png",
            "--max-colours=5",
            "--best-against-bg=#fff",
        );

        result.success().stdout("#693900\n").stderr("");
    }

    #[test]
    fn it_prints_the_version() {
        let result = run_command!("--version");

        // Match strings like `dominant_colours 1.2.3`
        let is_version_string =
            predicate::str::is_match(r"^dominant_colours [0-9]+\.[0-9]+\.[0-9]+\n$").unwrap();

        result.success().stdout(is_version_string).stderr("");
    }
}

#[cfg(test)]
#[macro_use]
mod test_helpers {
    /// Run this command-line tool with zero or more arguments:
    ///
    ///     run_command!();
    ///     run_command!("shape.png");
    ///     run_command!("shape.png", "--sides=4", "--colour=red");
    ///
    /// This returns an `assert_cmd::assert::Assert` that will allow
    /// you to make assertions about the output.
    /// See https://docs.rs/assert_cmd/latest/assert_cmd/assert/struct.Assert.html
    #[macro_export]
    macro_rules! run_command {
        () => {
            assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME"))
                       .unwrap()
                       .assert()
        };

        ($($arg:expr),+ $(,)?) => {{
            assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME"))
                       .unwrap()
                       .args(&[$($arg),*])
                       .assert()
        }};
    }
}
