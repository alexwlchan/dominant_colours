use kmeans_colors::get_kmeans_hamerly;
use palette::{color_difference::Wcag21RelativeContrast, FromColor, Lab, Srgb};

pub fn find_dominant_colors(lab: &Vec<Lab>, max_colors: usize) -> Vec<Lab> {
    // This is based on code from the kmeans-colors binary, but with a bunch of
    // the options stripped out.
    // See https://github.com/okaneco/kmeans-colors/blob/0.5.0/src/bin/kmeans_colors/app.rs
    let max_iterations = 20;
    let converge = 1.0;
    let verbose = false;
    let seed: u64 = 0;

    assert!(lab.len() > 0);

    let result = get_kmeans_hamerly(max_colors, max_iterations, converge, verbose, lab, seed);

    result.centroids
}

pub fn choose_best_color_for_bg(colors: Vec<Lab>, background: &Srgb<u8>) -> Vec<Lab> {
    // Start by adding black and white to the list of candidate colors.
    //
    // They're boring, but any background colour will always have sufficient
    // contrast with at least one of them.
    let black = Srgb::new(0.0, 0.0, 0.0);
    let white = Srgb::new(1.0, 1.0, 1.0);

    // I suspect this is not the most "technically correct" way to convert
    // an Srgb<u8> to a Srgb<f32>, but it's good enough for my purposes.
    let mut extended_colors: Vec<Srgb<f32>> =
        colors.iter().map(|c| Srgb::<f32>::from_color(*c)).collect();

    extended_colors.push(black);
    extended_colors.push(white);

    let background: Srgb<f32> = Srgb::new(
        background.red as f32 / 255.0,
        background.green as f32 / 255.0,
        background.blue as f32 / 255.0,
    );

    // Filter for colors which meet the min contrast ratio
    let allowed_colors: Vec<Srgb<f32>> = extended_colors
        .into_iter()
        .filter(|c| background.has_min_contrast_graphics(*c))
        .collect();

    // Now pick the color with the highest saturation among the remaining.
    let best_color: Srgb<f32> = allowed_colors
        .into_iter()
        .max_by(|color_a, color_b| {
            saturation(color_a)
                .partial_cmp(&saturation(color_b))
                .unwrap()
        })
        .unwrap();

    vec![Lab::from_color(best_color)]
}

// Based on https://filmentor.academy/en/blogs/news/die-wunderbare-welt-der-mathematik-fur-farben
fn saturation(c: &Srgb<f32>) -> f32 {
    let min_rgb: f32 = vec![c.red, c.green, c.blue]
        .into_iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let max_rgb: f32 = vec![c.red, c.green, c.blue]
        .into_iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    if min_rgb == max_rgb {
        return 0.0;
    }

    let luminosity: f32 = 0.5 * (min_rgb + max_rgb);

    if luminosity == 1.0 {
        0.0
    } else {
        (max_rgb - min_rgb) / (1.0 - (2.0 * luminosity - 1.0).abs())
    }
}
