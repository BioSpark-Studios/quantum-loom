//! FORGE UI DESIGNER — Library · Modules · Panel · Layouts · Node Graph.
//! The stages are ported in the next pass; until then each shows what it is.

use crate::app::{ForgeApp, STAGES};
use crate::shell;
use egui::{Rect, Ui};

pub fn body(app: &mut ForgeApp, ui: &mut Ui, area: Rect) {
    let (_, icon, name, tip) = STAGES
        .iter()
        .find(|s| s.0 == app.prefs.stage)
        .copied()
        .unwrap_or(STAGES[0]);
    shell::stub(
        ui,
        area,
        icon,
        &format!("FORGE · {name}"),
        tip,
        "ARRIVES IN THE NEXT PASS · the Main Rack and Routing Matrix are live now",
    );
}

/// Stage tools shown in the workspace row.
pub fn tools(_app: &mut ForgeApp, _ui: &mut Ui, _compact: bool) {}
