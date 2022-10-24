#[derive(Debug)]
pub struct Action {
    pub path: String,
    pub no_palette: bool,
    pub max_colours: usize,
    pub options: ActionOptions,
}

#[derive(Debug)]
pub enum ActionOptions {
    GetDominantColours,
    GetBestColourWith { compared_to: palette::Srgb },
}
