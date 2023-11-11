use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(
    help_template = "{author-with-newline}{name} {version} {about-section}\n {usage-heading} {usage} \n {all-args} {tab}"
)]
pub struct Cli {
    #[arg(short, long, value_name = "N")]
    /// number of rays per pixel. Default is 10
    pub samples_per_pixel: Option<usize>,

    #[arg(short, long, value_name = "R")]
    /// image aspect ratio. Default is 16/9
    pub aspect_ratio: Option<f32>,

    #[arg(short, long, value_name = "W")]
    /// image width, in pixels. Default is 400
    pub width: Option<usize>,

    #[arg(short, long, value_name = "D")]
    /// Max number of generated secondary rays. Default is 10
    pub max_depth: Option<usize>,

    #[arg(short, long)]
    /// Display camera information
    pub dump_info: bool,
}
