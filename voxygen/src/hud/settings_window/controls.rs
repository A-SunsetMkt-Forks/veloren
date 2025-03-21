use super::{RESET_BUTTONS_HEIGHT, RESET_BUTTONS_WIDTH};

use crate::{
    GlobalState,
    game_input::GameInput,
    hud::{ERROR_COLOR, TEXT_BIND_CONFLICT_COLOR, TEXT_COLOR, img_ids::Imgs},
    session::settings_change::{Control as ControlChange, Control::*},
    ui::fonts::Fonts,
    window::MenuInput,
};
use conrod_core::{
    Borderable, Colorable, Labelable, Positionable, Sizeable, Widget, WidgetCommon, color,
    position::Relative,
    widget::{self, Button, DropDownList, Rectangle, Scrollbar, Text},
    widget_ids,
};
use i18n::Localization;
use std::sync::LazyLock;
use strum::IntoEnumIterator;

widget_ids! {
    struct Ids {
        window,
        window_r,
        window_scrollbar,
        reset_controls_button,
        keybind_helper,
        gamepad_mode_button,
        gamepad_option_dropdown,
        controls_alignment_rectangle,
        controls_texts[],
        controls_buttons[],
    }
}

#[derive(WidgetCommon)]
pub struct Controls<'a> {
    global_state: &'a GlobalState,
    imgs: &'a Imgs,
    fonts: &'a Fonts,
    localized_strings: &'a Localization,
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
}
impl<'a> Controls<'a> {
    pub fn new(
        global_state: &'a GlobalState,
        imgs: &'a Imgs,
        fonts: &'a Fonts,
        localized_strings: &'a Localization,
    ) -> Self {
        Self {
            global_state,
            imgs,
            fonts,
            localized_strings,
            common: widget::CommonBuilder::default(),
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum BindingMode {
    Keyboard,
    Gamepad,
}
#[derive(Clone, Copy)]
pub enum GamepadBindingOption {
    GameButtons,
    GameLayers,
    MenuButtons,
}

pub struct State {
    ids: Ids,
    pub binding_mode: BindingMode,
    pub gamepad_binding_option: GamepadBindingOption,
}

static SORTED_GAMEINPUTS: LazyLock<Vec<GameInput>> = LazyLock::new(|| {
    let mut bindings_vec: Vec<GameInput> = GameInput::iter().collect();
    bindings_vec.sort();
    bindings_vec
});
static SORTED_MENUINPUTS: LazyLock<Vec<MenuInput>> = LazyLock::new(|| {
    let mut bindings_vec: Vec<MenuInput> = MenuInput::iter().collect();
    bindings_vec.sort();
    bindings_vec
});

impl Widget for Controls<'_> {
    type Event = Vec<ControlChange>;
    type State = State;
    type Style = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
            binding_mode: BindingMode::Keyboard,
            gamepad_binding_option: GamepadBindingOption::GameButtons,
        }
    }

    fn style(&self) -> Self::Style {}

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        common_base::prof_span!("Controls::update");
        let widget::UpdateArgs { state, ui, .. } = args;

        let mut events = Vec::new();
        let key_layout = &self.global_state.window.key_layout;

        Rectangle::fill_with(args.rect.dim(), color::TRANSPARENT)
            .xy(args.rect.xy())
            .graphics_for(args.id)
            .scroll_kids()
            .scroll_kids_vertically()
            .set(state.ids.window, ui);
        Rectangle::fill_with([args.rect.w() / 2.0, args.rect.h()], color::TRANSPARENT)
            .top_right()
            .parent(state.ids.window)
            .set(state.ids.window_r, ui);
        Scrollbar::y_axis(state.ids.window)
            .thickness(5.0)
            .rgba(0.33, 0.33, 0.33, 1.0)
            .set(state.ids.window_scrollbar, ui);

        // These temporary variables exist so state is only borrowed by resize_ids.
        let binding_mode = state.binding_mode;
        let gamepad_binding_option = state.gamepad_binding_option;

        // Button and Text resizing logic to be used by each binding type branch
        let mut resize_ids = |len| {
            if len > state.ids.controls_texts.len() || len > state.ids.controls_buttons.len() {
                state.update(|s| {
                    s.ids
                        .controls_texts
                        .resize(len, &mut ui.widget_id_generator());
                    s.ids
                        .controls_buttons
                        .resize(len, &mut ui.widget_id_generator());
                });
            }
        };

        // Used for sequential placement in a flow-down pattern
        let mut previous_element_id = None;

        if let BindingMode::Gamepad = binding_mode {
            match gamepad_binding_option {
                GamepadBindingOption::GameButtons => {
                    let gamepad_controls = &self.global_state.window.controller_settings;

                    resize_ids(SORTED_GAMEINPUTS.len());

                    // Loop all existing keybindings and the ids for text and button widgets
                    for (game_input, (&text_id, &button_id)) in SORTED_GAMEINPUTS.iter().zip(
                        state
                            .ids
                            .controls_texts
                            .iter()
                            .zip(state.ids.controls_buttons.iter()),
                    ) {
                        let (input_string, input_color) =
                            // TODO: handle rebind text
                            if let Some(button) = gamepad_controls.get_game_button_binding(*game_input) {
                                (
                                    format!(
                                        "{} {}",
                                        button.display_string(self.localized_strings),
                                        button.try_shortened()
                                            .map_or("".to_owned(), |short| format!("({})", short))
                                    ),
                                    if gamepad_controls.game_button_has_conflicting_bindings(button) {
                                        TEXT_BIND_CONFLICT_COLOR
                                    } else {
                                        TEXT_COLOR
                                    },
                                )
                            } else {
                                (
                                    self.localized_strings
                                        .get_msg("hud-settings-unbound")
                                        .into_owned(),
                                    ERROR_COLOR,
                                )
                            };
                        let loc_key = self
                            .localized_strings
                            .get_msg(game_input.get_localization_key());
                        let text_widget = Text::new(&loc_key)
                            .color(TEXT_COLOR)
                            .font_id(self.fonts.cyri.conrod_id)
                            .font_size(self.fonts.cyri.scale(18));
                        let button_widget = Button::new()
                            .label(&input_string)
                            .label_color(input_color)
                            .label_font_id(self.fonts.cyri.conrod_id)
                            .label_font_size(self.fonts.cyri.scale(15))
                            .w(150.0)
                            .rgba(0.0, 0.0, 0.0, 0.0)
                            .border_rgba(0.0, 0.0, 0.0, 255.0)
                            .label_y(Relative::Scalar(3.0));
                        // Place top-left if it's the first text, else under the previous one
                        let text_widget = match previous_element_id {
                            None => {
                                text_widget.top_left_with_margins_on(state.ids.window, 10.0, 5.0)
                            },
                            Some(prev_id) => text_widget.down_from(prev_id, 10.0),
                        };
                        let text_width = text_widget.get_w(ui).unwrap_or(0.0);
                        text_widget.set(text_id, ui);
                        if button_widget
                            .right_from(text_id, 350.0 - text_width)
                            .set(button_id, ui)
                            .was_clicked()
                        {
                            // TODO: handle change and remove binding
                        }
                        // Set the previous id to the current one for the next cycle
                        previous_element_id = Some(text_id);
                    }
                },
                GamepadBindingOption::GameLayers => {
                    let gamepad_controls = &self.global_state.window.controller_settings;

                    resize_ids(SORTED_GAMEINPUTS.len());

                    // Loop all existing keybindings and the ids for text and button widgets
                    for (game_input, (&text_id, &button_id)) in SORTED_GAMEINPUTS.iter().zip(
                        state
                            .ids
                            .controls_texts
                            .iter()
                            .zip(state.ids.controls_buttons.iter()),
                    ) {
                        let (input_string, input_color) =
                            // TODO: handle rebind text
                            if let Some(entry) = gamepad_controls.get_layer_button_binding(*game_input) {
                                (
                                    entry.display_string(self.localized_strings),
                                    if gamepad_controls.layer_entry_has_conflicting_bindings(entry) {
                                        TEXT_BIND_CONFLICT_COLOR
                                    } else {
                                        TEXT_COLOR
                                    },
                                )
                            } else {
                                (
                                    self.localized_strings
                                        .get_msg("hud-settings-unbound")
                                        .into_owned(),
                                    ERROR_COLOR,
                                )
                            };
                        let loc_key = self
                            .localized_strings
                            .get_msg(game_input.get_localization_key());
                        let text_widget = Text::new(&loc_key)
                            .color(TEXT_COLOR)
                            .font_id(self.fonts.cyri.conrod_id)
                            .font_size(self.fonts.cyri.scale(18));
                        let button_widget = Button::new()
                            .label(&input_string)
                            .label_color(input_color)
                            .label_font_id(self.fonts.cyri.conrod_id)
                            .label_font_size(self.fonts.cyri.scale(15))
                            .w(150.0)
                            .rgba(0.0, 0.0, 0.0, 0.0)
                            .border_rgba(0.0, 0.0, 0.0, 255.0)
                            .label_y(Relative::Scalar(3.0));
                        // Place top-left if it's the first text, else under the previous one
                        let text_widget = match previous_element_id {
                            None => {
                                text_widget.top_left_with_margins_on(state.ids.window, 10.0, 5.0)
                            },
                            Some(prev_id) => text_widget.down_from(prev_id, 10.0),
                        };
                        let text_width = text_widget.get_w(ui).unwrap_or(0.0);
                        text_widget.set(text_id, ui);
                        if button_widget
                            .right_from(text_id, 350.0 - text_width)
                            .set(button_id, ui)
                            .was_clicked()
                        {
                            // TODO: handle change and remove binding
                        }
                        // Set the previous id to the current one for the next cycle
                        previous_element_id = Some(text_id);
                    }
                },
                GamepadBindingOption::MenuButtons => {
                    let gamepad_controls = &self.global_state.window.controller_settings;

                    resize_ids(SORTED_MENUINPUTS.len());

                    // Loop all existing keybindings and the ids for text and button widgets
                    for (menu_input, (&text_id, &button_id)) in SORTED_MENUINPUTS.iter().zip(
                        state
                            .ids
                            .controls_texts
                            .iter()
                            .zip(state.ids.controls_buttons.iter()),
                    ) {
                        let (input_string, input_color) =
                            // TODO: handle rebind text
                            if let Some(button) = gamepad_controls.get_menu_button_binding(*menu_input) {
                                (
                                    format!(
                                        "{} {}",
                                        button.display_string(self.localized_strings),
                                        button.try_shortened()
                                            .map_or("".to_owned(), |short| format!("({})", short))
                                    ),
                                    if gamepad_controls.menu_button_has_conflicting_bindings(button) {
                                        TEXT_BIND_CONFLICT_COLOR
                                    } else {
                                        TEXT_COLOR
                                    },
                                )
                            } else {
                                (
                                    self.localized_strings
                                        .get_msg("hud-settings-unbound")
                                        .into_owned(),
                                    ERROR_COLOR,
                                )
                            };
                        let loc_key = self
                            .localized_strings
                            .get_msg(menu_input.get_localization_key());
                        let text_widget = Text::new(&loc_key)
                            .color(TEXT_COLOR)
                            .font_id(self.fonts.cyri.conrod_id)
                            .font_size(self.fonts.cyri.scale(18));
                        let button_widget = Button::new()
                            .label(&input_string)
                            .label_color(input_color)
                            .label_font_id(self.fonts.cyri.conrod_id)
                            .label_font_size(self.fonts.cyri.scale(15))
                            .w(150.0)
                            .rgba(0.0, 0.0, 0.0, 0.0)
                            .border_rgba(0.0, 0.0, 0.0, 255.0)
                            .label_y(Relative::Scalar(3.0));
                        // Place top-left if it's the first text, else under the previous one
                        let text_widget = match previous_element_id {
                            None => {
                                text_widget.top_left_with_margins_on(state.ids.window, 10.0, 5.0)
                            },
                            Some(prev_id) => text_widget.down_from(prev_id, 10.0),
                        };
                        let text_width = text_widget.get_w(ui).unwrap_or(0.0);
                        text_widget.set(text_id, ui);
                        if button_widget
                            .right_from(text_id, 350.0 - text_width)
                            .set(button_id, ui)
                            .was_clicked()
                        {
                            // TODO: handle change and remove binding
                        }
                        // Set the previous id to the current one for the next cycle
                        previous_element_id = Some(text_id);
                    }
                },
            }
        } else {
            let controls = &self.global_state.settings.controls;

            resize_ids(SORTED_GAMEINPUTS.len());

            // Loop all existing keybindings and the ids for text and button widgets
            for (game_input, (&text_id, &button_id)) in SORTED_GAMEINPUTS.iter().zip(
                state
                    .ids
                    .controls_texts
                    .iter()
                    .zip(state.ids.controls_buttons.iter()),
            ) {
                let (key_string, key_color) =
                    if self.global_state.window.remapping_keybindings == Some(*game_input) {
                        (
                            self.localized_strings
                                .get_msg("hud-settings-awaitingkey")
                                .into_owned(),
                            TEXT_COLOR,
                        )
                    } else if let Some(key) = controls.get_binding(*game_input) {
                        (
                            format!(
                                "{} {}",
                                key.display_string(key_layout),
                                key.try_shortened(key_layout)
                                    .map_or("".to_owned(), |short| format!("({})", short))
                            ),
                            if controls.has_conflicting_bindings(key) {
                                TEXT_BIND_CONFLICT_COLOR
                            } else {
                                TEXT_COLOR
                            },
                        )
                    } else {
                        (
                            self.localized_strings
                                .get_msg("hud-settings-unbound")
                                .into_owned(),
                            ERROR_COLOR,
                        )
                    };
                let loc_key = self
                    .localized_strings
                    .get_msg(game_input.get_localization_key());
                let text_widget = Text::new(&loc_key)
                    .color(TEXT_COLOR)
                    .font_id(self.fonts.cyri.conrod_id)
                    .font_size(self.fonts.cyri.scale(18));
                let button_widget = Button::new()
                    .label(&key_string)
                    .label_color(key_color)
                    .label_font_id(self.fonts.cyri.conrod_id)
                    .label_font_size(self.fonts.cyri.scale(15))
                    .w(150.0)
                    .rgba(0.0, 0.0, 0.0, 0.0)
                    .border_rgba(0.0, 0.0, 0.0, 255.0)
                    .label_y(Relative::Scalar(3.0));
                // Place top-left if it's the first text, else under the previous one
                let text_widget = match previous_element_id {
                    None => text_widget.top_left_with_margins_on(state.ids.window, 10.0, 5.0),
                    Some(prev_id) => text_widget.down_from(prev_id, 10.0),
                };
                let text_width = text_widget.get_w(ui).unwrap_or(0.0);
                text_widget.set(text_id, ui);
                button_widget
                    .right_from(text_id, 350.0 - text_width)
                    .set(button_id, ui);

                for _ in ui.widget_input(button_id).clicks().left() {
                    events.push(ChangeBinding(*game_input));
                }
                for _ in ui.widget_input(button_id).clicks().right() {
                    events.push(RemoveBinding(*game_input));
                }
                // Set the previous id to the current one for the next cycle
                previous_element_id = Some(text_id);
            }
        }

        // Reset the KeyBindings settings to the default settings
        if let Some(prev_id) = previous_element_id {
            if Button::image(self.imgs.button)
                .w_h(RESET_BUTTONS_WIDTH, RESET_BUTTONS_HEIGHT)
                .hover_image(self.imgs.button_hover)
                .press_image(self.imgs.button_press)
                .down_from(prev_id, 20.0)
                .label(
                    &self
                        .localized_strings
                        .get_msg("hud-settings-reset_keybinds"),
                )
                .label_font_size(self.fonts.cyri.scale(14))
                .label_color(TEXT_COLOR)
                .label_font_id(self.fonts.cyri.conrod_id)
                .label_y(Relative::Scalar(2.0))
                .set(state.ids.reset_controls_button, ui)
                .was_clicked() &&
                // TODO: handle reset button in gamepad mode
                state.binding_mode != BindingMode::Gamepad
            {
                events.push(ResetKeyBindings);
            }
            previous_element_id = Some(state.ids.reset_controls_button)
        }

        let offset = ui
            .widget_graph()
            .widget(state.ids.window)
            .and_then(|widget| {
                widget
                    .maybe_y_scroll_state
                    .as_ref()
                    .map(|scroll| scroll.offset)
            })
            .unwrap_or(0.0);

        let keybind_helper_text = self
            .localized_strings
            .get_msg("hud-settings-keybind-helper");
        let keybind_helper = Text::new(&keybind_helper_text)
            .color(TEXT_COLOR)
            .font_id(self.fonts.cyri.conrod_id)
            .font_size(self.fonts.cyri.scale(18));
        keybind_helper
            .top_right_with_margins_on(state.ids.window, offset + 5.0, 10.0)
            .set(state.ids.keybind_helper, ui);

        if let BindingMode::Gamepad = state.binding_mode {
            let game_buttons = &self.localized_strings.get_msg("hud-settings-game_buttons");
            let game_layers = &self.localized_strings.get_msg("hud-settings-game_layers");
            let menu_buttons = &self.localized_strings.get_msg("hud-settings-menu_buttons");

            let binding_mode_list = [game_buttons, game_layers, menu_buttons];
            if let Some(clicked) = DropDownList::new(
                &binding_mode_list,
                Some(state.gamepad_binding_option as usize),
            )
            .label_color(TEXT_COLOR)
            .label_font_id(self.fonts.cyri.conrod_id)
            .label_font_size(self.fonts.cyri.scale(15))
            .w(125.0)
            .rgba(0.0, 0.0, 0.0, 0.0)
            .border_rgba(0.0, 0.0, 0.0, 255.0)
            .label_y(Relative::Scalar(1.0))
            .down_from(state.ids.gamepad_mode_button, 10.0)
            .set(state.ids.gamepad_option_dropdown, ui)
            {
                match clicked {
                    0 => {
                        state.update(|s| {
                            s.gamepad_binding_option = GamepadBindingOption::GameButtons
                        });
                    },
                    1 => {
                        state.update(|s| {
                            s.gamepad_binding_option = GamepadBindingOption::GameLayers
                        });
                    },
                    2 => {
                        state.update(|s| {
                            s.gamepad_binding_option = GamepadBindingOption::MenuButtons
                        });
                    },
                    _ => {
                        state.update(|s| {
                            s.gamepad_binding_option = GamepadBindingOption::GameButtons
                        });
                    },
                }
            }
        }

        let gamepad = &self.localized_strings.get_msg("hud-settings-gamepad");
        let keyboard = &self.localized_strings.get_msg("hud-settings-keyboard");

        let binding_mode_toggle_widget = Button::new()
            .label(if let BindingMode::Gamepad = state.binding_mode {
                gamepad
            } else {
                keyboard
            })
            .label_color(TEXT_COLOR)
            .label_font_id(self.fonts.cyri.conrod_id)
            .label_font_size(self.fonts.cyri.scale(15))
            .w(125.0)
            .rgba(0.0, 0.0, 0.0, 0.0)
            .border_rgba(0.0, 0.0, 0.0, 255.0)
            .label_y(Relative::Scalar(1.0));
        if binding_mode_toggle_widget
            .down_from(state.ids.keybind_helper, 10.0)
            .align_right_of(state.ids.keybind_helper)
            .set(state.ids.gamepad_mode_button, ui)
            .was_clicked()
        {
            if let BindingMode::Keyboard = state.binding_mode {
                state.update(|s| s.binding_mode = BindingMode::Gamepad);
            } else {
                state.update(|s| s.binding_mode = BindingMode::Keyboard);
            }
        }

        // Add an empty text widget to simulate some bottom margin, because conrod sucks
        if let Some(prev_id) = previous_element_id {
            Rectangle::fill_with([1.0, 1.0], color::TRANSPARENT)
                .down_from(prev_id, 10.0)
                .set(state.ids.controls_alignment_rectangle, ui);
        }

        events
    }
}
