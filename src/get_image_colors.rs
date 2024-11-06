// This file exports a single function, which is used to read the
// pixel data from an image.
//
// This includes resizing the image to a smaller size (~400×400) for
// faster downstream computations.
//
// It returns a Vec<Lab>, which can be passed to the k-means process.

use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use image::codecs::gif::GifDecoder;
use image::codecs::webp::WebPDecoder;
use image::imageops::FilterType;
use image::{AnimationDecoder, DynamicImage, Frame, ImageFormat};
use palette::cast::from_component_slice;
use palette::{IntoColor, Lab, Srgba};

pub fn get_image_colors(path: &PathBuf) -> Result<Vec<Lab>, GetImageColorsErr> {
    let format = get_format(path)?;

    let f = File::open(path)?;
    let reader = BufReader::new(f);

    let image_bytes = match format {
        ImageFormat::Gif => {
            let decoder = GifDecoder::new(reader)?;
            get_bytes_for_animated_image(decoder)
        }

        ImageFormat::WebP if is_animated_webp(path) => {
            let decoder = WebPDecoder::new(reader)?;
            get_bytes_for_animated_image(decoder)
        }

        format => {
            let decoder = image::load(reader, format)?;
            get_bytes_for_static_image(decoder)
        }
    };

    let lab: Vec<Lab> = from_component_slice::<Srgba<u8>>(&image_bytes)
        .iter()
        .map(|x| x.into_format::<_, f32>().into_color())
        .collect();

    Ok(lab)
}

#[derive(Debug)]
pub enum GetImageColorsErr {
    IoError(std::io::Error),
    ImageError(image::ImageError),
    GetFormatError(String),
}

impl Display for GetImageColorsErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetImageColorsErr::IoError(io_error) => write!(f, "{}", io_error),
            GetImageColorsErr::ImageError(image_error) => write!(f, "{}", image_error),
            GetImageColorsErr::GetFormatError(format_error) => write!(f, "{}", format_error),
        }
    }
}

impl From<std::io::Error> for GetImageColorsErr {
    fn from(e: std::io::Error) -> GetImageColorsErr {
        return GetImageColorsErr::IoError(e);
    }
}

impl From<image::ImageError> for GetImageColorsErr {
    fn from(e: image::ImageError) -> GetImageColorsErr {
        return GetImageColorsErr::ImageError(e);
    }
}

fn get_format(path: &PathBuf) -> Result<ImageFormat, GetImageColorsErr> {
    let format = match path.extension() {
        Some(ext) => Ok(image::ImageFormat::from_extension(ext)),
        None => Err(GetImageColorsErr::GetFormatError(
            "Path has no file extension, so could not determine image format".to_string(),
        )),
    };

    match format {
        Ok(Some(format)) => Ok(format),
        Ok(None) => Err(GetImageColorsErr::GetFormatError(
            "Unable to determine image format from file extension".to_string(),
        )),
        Err(e) => Err(e),
    }
}

/// Returns true if a WebP file is animated, and false otherwise.
///
/// This function assumes that the file exists and can be opened.
fn is_animated_webp(path: &PathBuf) -> bool {
    let f = File::open(path).unwrap();
    let reader = BufReader::new(f);

    match image_webp::WebPDecoder::new(reader) {
        // num_frames() returns the number of frames in the animation,
        // or zero if the image is not animated.
        // See https://docs.rs/image-webp/0.2.0/image_webp/struct.WebPDecoder.html#method.num_frames
        Ok(decoder) => decoder.num_frames() > 0,
        Err(_) => false,
    }
}

fn get_bytes_for_static_image(img: DynamicImage) -> Vec<u8> {
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

fn get_bytes_for_animated_image<'a>(decoder: impl AnimationDecoder<'a>) -> Vec<u8> {
    let frames: Vec<Frame> = decoder.into_frames().collect_frames().unwrap();
    assert!(frames.len() > 0);

    // If the image is animated, we want to make sure we look at multiple
    // frames when choosing the dominant colour.
    //
    // We don't want to pass all the frames to the k-means analysis, because
    // that would be incredibly memory-intensive and is unnecessary -- see
    // previous comments about wanting a good enough answer quickly.
    //
    // For that reason, we select a sample of up to 50 frames and use those
    // as the basis for analysis.
    //
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

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::get_image_colors::get_image_colors;

    // This image comes from https://stacks.wellcomecollection.org/peering-through-mri-scans-of-fruit-and-veg-part-1-a2e8b07bde6f
    //
    // I don't remember how I got these images, but for some reason they
    // caused v1.1.2 to fall over.  This is a test that they can still be
    // processed correctly.
    #[test]
    fn it_gets_colors_for_animated_gif() {
        let colors = get_image_colors(&PathBuf::from("./src/tests/garlic.gif"));

        assert!(colors.is_ok());
        assert!(colors.unwrap().len() > 0);
    }

    #[test]
    fn get_colors_for_static_gif() {
        let colors = get_image_colors(&PathBuf::from("./src/tests/yellow.gif"));

        assert!(colors.is_ok());
        assert!(colors.unwrap().len() > 0);
    }

    #[test]
    fn get_colors_for_static_webp() {
        let colors = get_image_colors(&PathBuf::from("./src/tests/purple.webp"));

        assert!(colors.is_ok());
        assert!(colors.unwrap().len() > 0);
    }

    #[test]
    fn get_colors_for_animated_webp() {
        let colors = get_image_colors(&PathBuf::from("./src/tests/animated_squares.webp"));

        assert!(colors.is_ok());
        assert!(colors.unwrap().len() > 0);
    }
}
