use crate::{SelectedEntityInfo, widgets::two_col_row};
use common::{
    comp::CharacterState,
    states::{charged_melee, dash_melee, leap_melee},
};
use egui::{Grid, Ui};

pub fn draw_char_state_group(
    ui: &mut Ui,
    _selected_entity_info: &SelectedEntityInfo,
    character_state: &CharacterState,
) {
    ui.horizontal(|ui| {
        ui.label("Current State: ");
        ui.label(character_state.to_string());
    });
    match character_state {
        CharacterState::DashMelee(data) => dash_melee_grid(ui, data),
        CharacterState::ChargedMelee(data) => charged_melee_grid(ui, data),
        // Character states with no associated data to display
        CharacterState::Dance
        | CharacterState::Idle(_)
        | CharacterState::Sit
        | CharacterState::GlideWield(_)
        | CharacterState::Talk(_)
        | CharacterState::Wielding(_) => {},
        CharacterState::LeapMelee(data) => leap_melee_grid(ui, data),
        _ => {
            ui.label("<Rendering not yet implemented for this state>");
        },
    };
}

fn charged_melee_grid(ui: &mut Ui, data: &charged_melee::Data) {
    Grid::new("selected_entity_charged_melee_grid")
        .spacing([40.0, 4.0])
        .max_col_width(100.0)
        .striped(true)
        .show(ui, |ui| #[rustfmt::skip] {
                two_col_row(ui, "Stage Section", data.stage_section.to_string());
                two_col_row(ui, "Timer", format!("{}ms", data.timer.as_millis()));
                two_col_row(ui, "Charge Amount", format!("{:.1}", data.charge_amount));
                two_col_row(ui, "Exhausted", if data.exhausted { "True" } else { "False" });
            });
}

fn dash_melee_grid(ui: &mut Ui, data: &dash_melee::Data) {
    Grid::new("selected_entity_dash_melee_grid")
        .spacing([40.0, 4.0])
        .max_col_width(100.0)
        .striped(true)
        .show(ui, |ui| #[rustfmt::skip] {
                two_col_row(ui, "Auto Charge", if data.auto_charge { "True" } else { "False " });
                two_col_row(ui, "Timer", format!("{}ms", data.timer.as_millis()));
                two_col_row(ui, "Stage Section", data.stage_section.to_string());
            });
}

fn leap_melee_grid(ui: &mut Ui, data: &leap_melee::Data) {
    Grid::new("selected_entity_leap_melee_grid")
        .spacing([40.0, 4.0])
        .max_col_width(100.0)
        .striped(true)
        .show(ui, |ui| #[rustfmt::skip] {
                two_col_row(ui, "Stage Section", data.stage_section.to_string());
                two_col_row(ui, "Timer", format!("{}ms", data.timer.as_millis()));
                two_col_row(ui, "Exhausted", if data.exhausted { "True" } else { "False " });
            });
}
