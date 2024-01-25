use clap::{Arg, ArgAction, Command};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn app() -> Command {
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
                .help("Just print the hex values, not colour previews")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("SEED")
                .long("seed")
                .help("Specify seed for picking the colours")
                .value_parser(value_parser!(u64))
                .default_value("0"),
        )
        .arg(
            Arg::new("random-seed")
                .long("random-seed")
                .help("Choose a random seed for picking the colours")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("terminal-colours")
                .long("terminal-colours")
                .help("Generate 16 colours for the terminal")
                .action(ArgAction::SetTrue),
        )
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
