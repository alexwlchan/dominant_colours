use std::str::FromStr;

use palette::Srgb;

pub enum ColourOutput {
    Hex,
    Rgb255,
    Rgb01,
}

impl ColourOutput {
    pub fn format_colour(&self, colour: &Srgb<u8>) -> String {
        match self {
            Self::Hex => format!("#{:02x}{:02x}{:02x}", colour.red, colour.green, colour.blue),
            Self::Rgb255 => format!("rgb({}, {}, {})", colour.red, colour.green, colour.blue),
            Self::Rgb01 => format!(
                "rgb({:.3}, {:.3}, {:.3})",
                colour.red as f32 / 255.0,
                colour.green as f32 / 255.0,
                colour.blue as f32 / 255.0
            ),
        }
    }
}

impl FromStr for ColourOutput {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hex" => Ok(Self::Hex),
            "rgb255" => Ok(Self::Rgb255),
            "rgb01" => Ok(Self::Rgb01),
            _ => Err(format!("'{}' is not a recognized output format. Accepted values are hex, rgb255, and rgb01.", s))
        }
    }
}
