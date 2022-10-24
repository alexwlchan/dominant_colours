use clap::{Arg, ArgAction, Command};
use palette::Srgb;
use std::str::FromStr;

use crate::models::{Action, ActionOptions};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn app() -> clap::Command {
    Command::new("dominant_colours")
        .version(VERSION)
        .author("Alex Chan <alex@alexwlchan.net>")
        .about("Find the dominant colours in an image")
        .arg(
            Arg::new("PATH")
                .help("path to the image to inspect")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("MAX-COLOURS")
                .long("max-colours")
                .help("how many colours to find")
                .value_parser(value_parser!(usize))
                .default_value("5"),
        )
        .arg(
            Arg::new("no-palette")
                .long("no-palette")
                .help("just print the hex values, not colour previews")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("COMPARED-TO")
                .long("compared-to")
                .help("find the colour from the image that looks best against this colour")
                .value_parser(Srgb::<u8>::from_str),
        )
}

pub fn parse_arguments(matches: clap::ArgMatches) -> Action {
    let path = matches
        .get_one::<String>("PATH")
        .expect("`path` is required")
        .to_owned();

    let no_palette = matches.get_flag("no-palette");

    let max_colours: usize = *matches
        .get_one::<usize>("MAX-COLOURS")
        .expect("`max-colours` is required");

    let compared_to = matches.get_one::<Srgb<u8>>("COMPARED-TO");

    let options = match compared_to {
        Some(compared_to) => ActionOptions::GetBestColourWith {
            compared_to: compared_to.to_owned(),
        },
        None => ActionOptions::GetDominantColours,
    };

    let command = Action {
        path,
        no_palette,
        max_colours,
        options,
    };

    command
}

#[cfg(test)]
mod tests {
    use crate::cli::app;

    // See https://github.com/clap-rs/clap/blob/master/CHANGELOG.md#300---2021-12-31
    #[test]
    fn verify_app() {
        app().debug_assert();
    }
}
