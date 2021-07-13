use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "books", about = "A toy payments engine.")]
pub struct Cli {
    /// Switch on verbosity.
    #[structopt(short, parse(from_occurrences))]
    pub verbosity: u8,

    /// Path to a csv file with transactions data.
    #[structopt(parse(from_os_str))]
    pub tx_csv_path: std::path::PathBuf,
}

pub fn from_args() -> Cli {
    Cli::from_args()
}