use kmeans_colors::get_kmeans_hamerly;
use palette::{FromColor, Lab, Srgb};

pub fn find_dominant_colors(lab: &Vec<Lab>, max_colors: usize) -> Vec<Srgb<u8>> {
    // This is based on code from the kmeans-colors binary, but with a bunch of
    // the options stripped out.
    // See https://github.com/okaneco/kmeans-colors/blob/0.5.0/src/bin/kmeans_colors/app.rs
    let max_iterations = 20;
    let converge = 1.0;
    let verbose = false;
    let seed: u64 = 0;

    let result = get_kmeans_hamerly(max_colors, max_iterations, converge, verbose, lab, seed);

    let rgb: Vec<Srgb<u8>> = result
        .centroids
        .iter()
        .map(|x| Srgb::from_color(*x).into_format())
        .collect::<Vec<Srgb<u8>>>();

    rgb
}
