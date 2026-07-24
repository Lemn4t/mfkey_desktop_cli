mod cli;

use std::path::PathBuf;

pub use cli::Cli;

pub enum Params {
    Run(RunParams),
    Auto(AutoParams),
}

pub struct RunParams {
    pub input_file: String,
    pub output_file: String,
    pub dict_output_dir: Option<String>,
    pub plain_ui: bool,
    pub accept_disclaimer: bool,
}

pub struct AutoParams {
    pub port: Option<String>,
    pub out_dir: Option<PathBuf>,
    pub plain_ui: bool,
    pub accept_disclaimer: bool,
}

pub fn parse() -> Params {
    use clap::Parser;
    let cli = Cli::parse();

    if cli.auto {
        return Params::Auto(AutoParams {
            port: cli.port,
            out_dir: cli.out,
            plain_ui: cli.plain,
            accept_disclaimer: cli.accept_disclaimer,
        });
    }

    Params::Run(RunParams {
        input_file: cli
            .input_file
            .expect("clap guarantees input_file is present when --auto is not set"),
        output_file: cli.output_file,
        dict_output_dir: cli.dict_output_dir,
        plain_ui: cli.plain,
        accept_disclaimer: cli.accept_disclaimer,
    })
}
