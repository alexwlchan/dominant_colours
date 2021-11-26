#![deny(warnings)]

#[macro_use]
extern crate clap;

use clap::{App, Arg};
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

    // Get the count as a number, or default to 5.
    // See https://github.com/clap-rs/clap/blob/v2.33.1/examples/12_typed_values.rs
    let count = value_t!(matches, "count", usize)
        .unwrap_or_else(|e| e.exit());

    let img = image::open(&path)
        .unwrap()
        .into_rgba8();

    let img_vec = img.into_raw();

    let lab: Vec<Lab> = Srgba::from_raw_slice(&img_vec)
        .iter()
        .map(|x| x.into_format().into())
        .collect();

    let max_iterations = 20;
    let converge = 50.0;
    let verbose = false;
    let seed: u64 = 0;

    let run_result = get_kmeans_hamerly(
        count,
        max_iterations,
        converge,
        verbose,
        &lab,
        seed,
    );

    let rgb = &run_result.centroids
        .iter()
        .map(|x| Srgb::from(*x).into_format())
        .collect::<Vec<Srgb<u8>>>();

    for c in rgb {
        println!("\x1B[38;2;{};{};{}m▇ #{:02x}{:02x}{:02x}\x1B[0m", c.red, c.green, c.blue, c.red, c.green, c.blue);
    }
}
