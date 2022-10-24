use clap::{App, Arg};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn app() -> clap::App<'static> {
    App::new("dominant_colours")
        .version(VERSION)
        .author("Alex Chan <alex@alexwlchan.net>")
        .about("Find the dominant colours in an image")
        .arg(
            Arg::with_name("PATH")
                .help("path to the image to inspect")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("MAX-COLOURS")
                .long("max-colours")
                .help("how many colours to find")
                .default_value("5")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("no-palette")
                .long("no-palette")
                .help("Just print the hex values, not colour previews")
                .takes_value(false),
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
