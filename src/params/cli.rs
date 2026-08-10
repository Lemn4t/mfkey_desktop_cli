use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "mfkey_desktop_cli")]
#[command(version)]
#[command(about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[arg(required_unless_present_any = ["auto", "ble"], value_name = "INPUT_FILE")]
    pub input_file: Option<String>,

    #[arg(default_value = "mf_classic_dict_user.nfc", value_name = "OUTPUT_FILE")]
    pub output_file: String,

    #[arg(value_name = "DICT_OUTPUT_DIR")]
    pub dict_output_dir: Option<String>,

    #[arg(long)]
    pub plain: bool,

    #[arg(long)]
    pub auto: bool,

    #[arg(long, conflicts_with = "auto")]
    pub ble: bool,

    #[arg(long, requires = "auto", value_name = "PORT")]
    pub port: Option<String>,

    #[arg(long, requires = "ble", value_name = "DEVICE_ID")]
    pub device: Option<String>,

    #[arg(long, value_name = "DIR")]
    pub out: Option<PathBuf>,

    #[arg(long)]
    pub accept_disclaimer: bool,
}
