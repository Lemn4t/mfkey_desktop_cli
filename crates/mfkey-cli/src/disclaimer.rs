use crate::config::Config;
use crate::ui::Ui;

const DISCLAIMER_TEXT: &str = "\
+----------------------------------------------------------------------+
|                             DISCLAIMER                               |
+----------------------------------------------------------------------+
| This tool is intended exclusively for security research, education,  |
| and testing on cards you own or have explicit permission to analyze. |
| The author is not responsible for any misuse. Use at your own risk   |
| and in accordance with the laws of your jurisdiction.                |
+----------------------------------------------------------------------+";

pub fn resolve_disclaimer_acceptance(ui: &Ui, force_accept: bool) -> bool {
    let mut config = Config::read();
    if config.disclaimer_accepted {
        return true;
    }

    if force_accept {
        if let Err(e) = config.accept_disclaimer_and_store() {
            ui.show_error(&format!("{e}"));
        }
        return true;
    }

    ui.show_disclaimer(DISCLAIMER_TEXT);
    let accepted = ui.confirm("Do you accept the terms of use?", false);

    if accepted && let Err(e) = config.accept_disclaimer_and_store() {
        ui.show_error(&format!("{e}"));
    }
    ui.show_pill_taken(accepted);

    accepted
}
