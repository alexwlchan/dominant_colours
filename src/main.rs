#[macro_use]
extern crate clap;

use clap::{App, Arg};
// use kmeans_colors::{get_kmeans_hamerly, MapColor};
use palette::{FromColor, IntoColor, Lab, Pixel, Srgb, Srgba};
use kmeans_colors::{get_kmeans, get_kmeans_hamerly, Calculate, Kmeans, MapColor, Sort};

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
    let converge = 5.0;
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
    //
    // let rgb = &run_result.centroids
    //     .iter()
    //     .map(|x| Srgb::from(*x).into_format())
    //     .collect::<Vec<Srgb<u8>>>();

    // println!("{:?}", run_result);

    println!("matches = {:?}", matches);
    println!("count = {:?}", count);
    // println!("lab = {:?}", lab);
    println!("Hello, world!");
}
