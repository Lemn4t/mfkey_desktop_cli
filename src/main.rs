mod auto;
mod core;
pub mod ext;
mod flipper;
mod generated_pb;
mod params;
mod run;
mod ui;

pub use generated_pb::*;

use crate::core::disclaimer::resolve_disclaimer_acceptance;
use crate::params::Params;
use crate::ui::{Ui, UiOptions};
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    let params = params::parse();
    let opts = UiOptions {
        mode: ui::resolve_output_mode(params.plain_ui()),
    };
    let ui = Ui::new(opts);
    if !resolve_disclaimer_acceptance(&ui, params.accept_disclaimer()) {
        return;
    }
    let ui = Arc::new(ui);
    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();
    let handler_ui = ui.clone();
    {
        if let Err(e) = ctrlc::set_handler(move || {
            handler_ui.show_interrupt();
            stop_clone.store(true, Ordering::SeqCst);
        }) {
            ui.show_error(&format!("Warning: failed to set Ctrl+C handler: {e}"));
        }
    }
    ui.show_title();

    match params {
        Params::Auto(params) => match auto::run_auto(ui.clone(), params, stop) {
            Ok(()) => process::exit(0),
            Err(e) => {
                ui.show_error(&format!("AUTO failed: {e}"));
                process::exit(1);
            }
        },
        Params::Run(params) => run::run(ui, params, stop),
    }
}
