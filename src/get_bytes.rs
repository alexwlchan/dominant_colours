use std::fs::File;

use image::codecs::gif::GifDecoder;
use image::imageops::FilterType;
use image::{AnimationDecoder, DynamicImage, Frame};

pub fn get_bytes_for_image(path: &str) -> Vec<u8> {
    let img = match image::open(&path) {
        Ok(im) => im,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
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
    let resized_img = img.resize(400, 400, FilterType::Nearest);

    resized_img.into_rgba8().into_raw()
}

pub fn get_bytes_for_gif(path: &str) -> Vec<u8> {
    let f = match File::open(path) {
        Ok(im) => im,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let decoder = GifDecoder::new(f).ok().unwrap();

    // If the GIF is animated, we want to make sure we look at multiple
    // frames when choosing the dominant colour.
    //
    // We don't want to pass all the frames to the k-means analysis, because
    // that would be incredibly memory-intensive and is unnecessary -- see
    // previous comments about wanting a good enough answer quickly.
    //
    // For that reason, we select a sample of up to 50 frames and use those
    // as the basis for analysis.
    let frames: Vec<Frame> = decoder.into_frames().collect_frames().unwrap();

    // How this works: it tells us we should be looking at the nth frame.
    // Examples:
    //
    //      frame count | nth frame | comment
    //      ------------+-----------+---------
    //      1           |     1     | in a 1-frame GIF, look at the only frame
    //      25          |     1     | look at every frame
    //      50          |     2     | look at every second frame
    //      78          |     3     | look at every third frame
    //
    // I'm sure there's a more idiomatic way to do this, but it was late
    // when I wrote this and it seems to work.
    //
    let nth_frame = if frames.len() <= 50 {
        1
    } else {
        ((frames.len() as f32) / (25 as f32)) as i32
    };

    let selected_frames = frames
        .iter()
        .enumerate()
        .filter(|(i, _)| (*i as f32 / nth_frame as f32).floor() == (*i as f32 / nth_frame as f32))
        .map(|(_, frame)| frame);

    // Now we go through the frames and extract all the pixels.  The k-means
    // process doesn't care about position, so we can concatenate the pixels
    // for each frame into one big Vec.
    //
    // As with non-GIF images, we resize the images down before loading them.
    // We resize to a smaller frame in GIFs because if there are multiple
    // frames, we don't care as much about individual frames, and we want
    // to avoid a large Vec<u8> in memory.
    let resize = if frames.len() == 1 { 400 } else { 100 };

    selected_frames
        .map(|frame| {
            DynamicImage::ImageRgba8(frame.buffer().clone())
                .resize(resize, resize, FilterType::Nearest)
                .into_rgba8()
                .into_raw()
        })
        .into_iter()
        .flatten()
        .collect()
}
