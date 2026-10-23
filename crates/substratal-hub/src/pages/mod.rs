//! Screens. Hand-built screens are listed in [`build`]; everything else
//! renders from the spec through [`scaffold`].

mod account;
mod scaffold;
mod settings;
mod users;

use std::rc::Rc;

use substratal_core::screens::Screen;

use crate::shell::Shell;

pub fn build(
    shell: &Rc<Shell>,
    screen: &'static Screen,
    params: &[(String, String)],
) -> gtk::Widget {
    let _ = params;
    match screen.id.as_str() {
        "USR-01" => users::page(shell, screen),
        "SET-01" => settings::page(shell, screen),
        "ACC-01" => account::page(shell, screen),
        _ => scaffold::page(shell, screen),
    }
}

/// Hand-built screens reuse the scaffold's header for consistency.
pub(crate) use scaffold::{content as scaffold_content, page as scaffold_page};
