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

pub enum AutoTransport {
    Usb { port: Option<String> },
    Ble { device: Option<String> },
}

pub struct AutoParams {
    pub transport: AutoTransport,
    pub out_dir: Option<PathBuf>,
    pub plain_ui: bool,
    pub accept_disclaimer: bool,
}

pub fn parse() -> Params {
    use clap::Parser;
    let cli = Cli::parse();

    if cli.auto || cli.ble {
        let transport = if cli.ble {
            AutoTransport::Ble { device: cli.device }
        } else {
            AutoTransport::Usb { port: cli.port }
        };
        return Params::Auto(AutoParams {
            transport,
            out_dir: cli.out,
            plain_ui: cli.plain,
            accept_disclaimer: cli.accept_disclaimer,
        });
    }

    Params::Run(RunParams {
        input_file: cli
            .input_file
            .expect("clap guarantees input_file is present when --auto/--ble are not set"),
        output_file: cli.output_file,
        dict_output_dir: cli.dict_output_dir,
        plain_ui: cli.plain,
        accept_disclaimer: cli.accept_disclaimer,
    })
}

impl Params {
    pub fn plain_ui(&self) -> bool {
        match self {
            Params::Run(p) => p.plain_ui,
            Params::Auto(p) => p.plain_ui,
        }
    }

    pub fn accept_disclaimer(&self) -> bool {
        match self {
            Params::Run(p) => p.accept_disclaimer,
            Params::Auto(p) => p.accept_disclaimer,
        }
    }
}
