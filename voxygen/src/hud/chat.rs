use super::{
    ChatTab, ERROR_COLOR, FACTION_COLOR, GROUP_COLOR, INFO_COLOR, KILL_COLOR, OFFLINE_COLOR,
    ONLINE_COLOR, REGION_COLOR, SAY_COLOR, TELL_COLOR, TEXT_COLOR, WORLD_COLOR, img_ids::Imgs,
};
use crate::{
    GlobalState,
    cmd::complete,
    settings::{ChatSettings, chat::MAX_CHAT_TABS},
    ui::{
        Scale,
        fonts::{Font, Fonts},
    },
};
use client::Client;
use common::{
    cmd::ServerChatCommand,
    comp::{ChatMode, ChatMsg, ChatType, group::Role},
};
use conrod_core::{
    Color, Colorable, Labelable, Positionable, Sizeable, Ui, UiCell, Widget, WidgetCommon, color,
    input::Key,
    position::Dimension,
    text::{
        self,
        cursor::{self, Index},
    },
    widget::{self, Button, Id, Image, Line, List, Rectangle, Text, TextEdit},
    widget_ids,
};
use i18n::Localization;
use i18n_helpers::localize_chat_message;
use std::collections::{HashSet, VecDeque};
use vek::{Vec2, approx::AbsDiffEq};

/// Determines whether a message is from a muted player.
///
/// These messages will not be shown anywhere and do not need to be retained in
/// chat queues.
pub fn is_muted(client: &Client, profile: &crate::profile::Profile, msg: &ChatMsg) -> bool {
    if let Some(uid) = msg.uid()
        && let Some(player_info) = client.player_list().get(&uid)
    {
        profile.mutelist.contains_key(&player_info.uuid)
    } else {
        false
    }
}

/// Determines whether a message will be sent to the chat box.
///
/// Some messages like NPC messages are only displayed as in-world chat bubbles.
pub fn show_in_chatbox(msg: &ChatMsg) -> bool {
    // Don't put NPC messages in chat box.
    !matches!(msg.chat_type, ChatType::Npc(_))
}

pub const MAX_MESSAGES: usize = 100;

/// Chat messages received from the client before entering the
/// `SessionState`.
///
/// We transfer these to HUD when it is displayed in `SessionState`.
///
/// Messages that aren't show in the chat box aren't retained (e.g. ones
/// that would just show as in-world chat bubbles).
#[derive(Default)]
pub struct MessageBacklog(pub(super) VecDeque<ChatMsg>);

impl MessageBacklog {
    pub fn new_message(
        &mut self,
        client: &Client,
        profile: &crate::profile::Profile,
        msg: ChatMsg,
    ) {
        if !is_muted(client, profile, &msg) && show_in_chatbox(&msg) {
            self.0.push_back(msg);
            if self.0.len() > MAX_MESSAGES {
                self.0.pop_front();
            }
        }
    }
}

const CHAT_ICON_WIDTH: f64 = 16.0;
const CHAT_MARGIN_THICKNESS: f64 = 2.0;
const CHAT_ICON_HEIGHT: f64 = 16.0;
const MIN_DIMENSION: Vec2<f64> = Vec2::new(400.0, 150.0);
const MAX_DIMENSION: Vec2<f64> = Vec2::new(650.0, 500.0);

const CHAT_TAB_HEIGHT: f64 = 20.0;
const CHAT_TAB_ALL_WIDTH: f64 = 40.0;

/*#[const_tweaker::tweak(min = 0.0, max = 60.0, step = 1.0)]
const X: f64 = 18.0;*/

widget_ids! {
    struct Ids {
        draggable_area,
        message_box,
        message_box_bg,
        chat_input,
        chat_input_bg,
        chat_input_icon,
        chat_input_border_up,
        chat_input_border_down,
        chat_input_border_left,
        chat_input_border_right,
        chat_arrow,
        chat_icon_align,
        chat_icons[],
        chat_badges[],

        chat_tab_align,
        chat_tab_all,
        chat_tab_selected,
        chat_tabs[],
        chat_tab_tooltip_bg,
        chat_tab_tooltip_text,
    }
}

#[derive(WidgetCommon)]
pub struct Chat<'a> {
    pulse: f32,
    new_messages: &'a mut VecDeque<ChatMsg>,
    client: &'a Client,
    force_input: Option<String>,
    force_cursor: Option<Index>,
    force_completions: Option<Vec<String>>,

    global_state: &'a GlobalState,
    imgs: &'a Imgs,
    fonts: &'a Fonts,

    #[conrod(common_builder)]
    common: widget::CommonBuilder,

    // TODO: add an option to adjust this
    history_max: usize,
    scale: Scale,

    localized_strings: &'a Localization,
    clear_messages: bool,
}

impl<'a> Chat<'a> {
    pub fn new(
        new_messages: &'a mut VecDeque<ChatMsg>,
        client: &'a Client,
        global_state: &'a GlobalState,
        pulse: f32,
        imgs: &'a Imgs,
        fonts: &'a Fonts,
        localized_strings: &'a Localization,
        scale: Scale,
        clear_messages: bool,
    ) -> Self {
        Self {
            pulse,
            new_messages,
            client,
            force_input: None,
            force_cursor: None,
            force_completions: None,
            imgs,
            fonts,
            global_state,
            common: widget::CommonBuilder::default(),
            history_max: 32,
            localized_strings,
            scale,
            clear_messages,
        }
    }

    pub fn prepare_tab_completion(mut self, input: String) -> Self {
        self.force_completions = if let Some(index) = input.find('\t') {
            Some(complete(
                &input[..index],
                self.client,
                self.localized_strings,
                &self.global_state.settings.chat.chat_cmd_prefix.to_string(),
            ))
        } else {
            None
        };
        self
    }

    pub fn input(mut self, input: String) -> Self {
        self.force_input = Some(input);
        self
    }

    pub fn cursor_pos(mut self, index: Index) -> Self {
        self.force_cursor = Some(index);
        self
    }

    pub fn scrolled_to_bottom(state: &State, ui: &UiCell) -> bool {
        // Might be more efficient to cache result and update it when a scroll event has
        // occurred instead of every frame.
        if let Some(scroll) = ui
            .widget_graph()
            .widget(state.ids.message_box)
            .and_then(|widget| widget.maybe_y_scroll_state)
        {
            scroll.offset + 50.0 >= scroll.offset_bounds.start
        } else {
            false
        }
    }
}

struct InputState {
    message: String,
    mode: ChatMode,
}

pub struct State {
    messages: VecDeque<ChatMsg>,
    input: InputState,
    ids: Ids,
    history: VecDeque<String>,
    // Index into the history Vec, history_pos == 0 is history not in use
    // otherwise index is history_pos -1
    history_pos: usize,
    completions: Vec<String>,
    // Index into the completion Vec
    completions_index: Option<usize>,
    // At which character is tab completion happening
    completion_cursor: Option<usize>,
    // last time mouse has been hovered
    tabs_last_hover_pulse: Option<f32>,
    // last chat_tab (used to see if chat tab has been changed)
    prev_chat_tab: Option<ChatTab>,
    //whether or not a scroll action is queued
    scroll_next: bool,
}

pub enum Event {
    TabCompletionStart(String),
    SendMessage(String),
    SendCommand(String, Vec<String>),
    Focus(Id),
    ChangeChatTab(Option<usize>),
    ShowChatTabSettings(usize),
    ResizeChat(Vec2<f64>),
    MoveChat(Vec2<f64>),
    DisableForceChat,
}

impl Widget for Chat<'_> {
    type Event = Vec<Event>;
    type State = State;
    type Style = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            input: InputState {
                message: "".to_owned(),
                mode: ChatMode::default(),
            },
            messages: VecDeque::new(),
            history: VecDeque::new(),
            history_pos: 0,
            completions: Vec::new(),
            completions_index: None,
            completion_cursor: None,
            ids: Ids::new(id_gen),
            tabs_last_hover_pulse: None,
            prev_chat_tab: None,
            scroll_next: false,
        }
    }

    fn style(&self) -> Self::Style {}

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        fn adjust_border_opacity(color: Color, opacity: f32) -> Color {
            match color {
                Color::Rgba(r, g, b, a) => Color::Rgba(r, g, b, (a + opacity) / 2.0),
                _ => panic!("Color input should be Rgba, instead found: {:?}", color),
            }
        }
        common_base::prof_span!("Chat::update");

        let widget::UpdateArgs { id, state, ui, .. } = args;

        let mut events = Vec::new();

        let chat_settings = &self.global_state.settings.chat;
        let force_chat = !(&self.global_state.settings.interface.toggle_chat);
        let chat_tabs = &chat_settings.chat_tabs;
        let current_chat_tab = chat_settings.chat_tab_index.and_then(|i| chat_tabs.get(i));
        let chat_size = Vec2::new(chat_settings.chat_size_x, chat_settings.chat_size_y);
        let chat_pos = Vec2::new(chat_settings.chat_pos_x, chat_settings.chat_pos_y);
        let chat_box_input_width = chat_size.x - CHAT_ICON_WIDTH - 12.0;

        if self.clear_messages {
            state.update(|s| s.messages.clear());
        }

        // Empty old messages
        state.update(|s| {
            while s.messages.len() > MAX_MESSAGES {
                s.messages.pop_front();
            }
        });

        let chat_in_screen_upper = chat_pos.y > ui.win_h / 2.0;

        let pos_delta: Vec2<f64> = ui
            .widget_input(state.ids.draggable_area)
            .drags()
            .left()
            .map(|drag| Vec2::<f64>::from(drag.delta_xy))
            .sum();
        let new_pos = (chat_pos + pos_delta).map(|e| e.max(0.)).map2(
            self.scale.scale_point(Vec2::new(ui.win_w, ui.win_h))
                - Vec2::unit_y() * CHAT_TAB_HEIGHT
                - chat_size,
            |e, bounds| e.min(bounds),
        );
        if new_pos.abs_diff_ne(&chat_pos, f64::EPSILON) {
            events.push(Event::MoveChat(new_pos));
        }
        let size_delta: Vec2<f64> = ui
            .widget_input(state.ids.draggable_area)
            .drags()
            .right()
            .map(|drag| Vec2::<f64>::from(drag.delta_xy))
            .sum();
        let new_size = (chat_size + size_delta)
            .map3(
                self.scale.scale_point(MIN_DIMENSION),
                self.scale.scale_point(MAX_DIMENSION),
                |sz, min, max| sz.clamp(min, max),
            )
            .map2(
                self.scale.scale_point(Vec2::new(ui.win_w, ui.win_h))
                    - Vec2::unit_y() * CHAT_TAB_HEIGHT
                    - new_pos,
                |e, bounds| e.min(bounds),
            );
        if new_size.abs_diff_ne(&chat_size, f64::EPSILON) {
            events.push(Event::ResizeChat(new_size));
        }

        // Maintain scrolling //
        if !self.new_messages.is_empty() {
            for message in self.new_messages.iter() {
                // Log the output of commands since the ingame terminal doesn't support copying
                // the output to the clipboard
                if let ChatType::CommandInfo = &message.chat_type {
                    tracing::info!("Chat command info: {:?}", message.content());
                }
            }
            //new messages - update chat w/ them & scroll down if at bottom of chat
            state.update(|s| s.messages.extend(self.new_messages.drain(..)));
            // Prevent automatic scroll upon new messages if not already scrolled to bottom
            if Self::scrolled_to_bottom(state, ui) {
                ui.scroll_widget(state.ids.message_box, [0.0, f64::MAX]);
            }
        }

        // Trigger scroll event queued from previous frame
        if state.scroll_next {
            ui.scroll_widget(state.ids.message_box, [0.0, f64::MAX]);
            state.update(|s| s.scroll_next = false);
        }

        // Queue scroll event if switching from a different tab
        if current_chat_tab != state.prev_chat_tab.as_ref() {
            state.update(|s| s.prev_chat_tab = current_chat_tab.cloned());
            state.update(|s| s.scroll_next = true); //make scroll happen only once any filters to the messages have already been applied
        }

        if let Some(comps) = &self.force_completions {
            state.update(|s| s.completions.clone_from(comps));
        }

        let mut force_cursor = self.force_cursor;

        // If up or down are pressed: move through history
        // If any key other than up, down, or tab is pressed: stop completion.
        let (history_dir, tab_dir, stop_tab_completion) =
            ui.widget_input(state.ids.chat_input).presses().key().fold(
                (0isize, 0isize, false),
                |(n, m, tc), key_press| match key_press.key {
                    Key::Up => (n + 1, m - 1, tc),
                    Key::Down => (n - 1, m + 1, tc),
                    Key::Tab => (n, m + 1, tc),
                    _ => (n, m, true),
                },
            );

        // Handle tab completion
        let request_tab_completions = if stop_tab_completion {
            // End tab completion
            state.update(|s| {
                if s.completion_cursor.is_some() {
                    s.completion_cursor = None;
                }
                s.completions_index = None;
            });
            false
        } else if let Some(cursor) = state.completion_cursor {
            // Cycle through tab completions of the current word
            if state.input.message.contains('\t') {
                state.update(|s| s.input.message.retain(|c| c != '\t'));
                //tab_dir + 1
            }
            if !state.completions.is_empty() && (tab_dir != 0 || state.completions_index.is_none())
            {
                state.update(|s| {
                    let len = s.completions.len();
                    s.completions_index = Some(
                        (s.completions_index.unwrap_or(0) + (tab_dir + len as isize) as usize)
                            % len,
                    );
                    if let Some(replacement) = &s.completions.get(s.completions_index.unwrap()) {
                        let (completed, offset) =
                            do_tab_completion(cursor, &s.input.message, replacement);
                        force_cursor = cursor_offset_to_index(
                            offset,
                            &completed,
                            ui,
                            self.fonts,
                            chat_box_input_width,
                        );
                        s.input.message = completed;
                    }
                });
            }
            false
        } else if let Some(cursor) = state.input.message.find('\t') {
            // Begin tab completion
            state.update(|s| s.completion_cursor = Some(cursor));
            true
        } else {
            // Not tab completing
            false
        };

        // Check if we need to change the chat mode if we have completed a command
        if state.input.message.ends_with(' ') {
            change_chat_mode(
                state.input.message.clone(),
                state,
                &mut events,
                chat_settings,
            );
        }

        // Move through history
        if history_dir != 0 && state.completion_cursor.is_none() {
            state.update(|s| {
                if history_dir > 0 {
                    if s.history_pos < s.history.len() {
                        s.history_pos += 1;
                    }
                } else if s.history_pos > 0 {
                    s.history_pos -= 1;
                }
                if let Some(before) = s.history.iter().nth_back(s.history.len() - s.history_pos) {
                    s.input.message.clone_from(before);
                    force_cursor = cursor_offset_to_index(
                        s.input.message.len(),
                        &s.input.message,
                        ui,
                        self.fonts,
                        chat_box_input_width,
                    );
                } else {
                    s.input.message.clear();
                }
            });
        }

        let keyboard_capturer = ui.global_input().current.widget_capturing_keyboard;

        if let Some(input) = &self.force_input {
            state.update(|s| s.input.message = input.to_string());
        }

        let input_focused =
            keyboard_capturer == Some(state.ids.chat_input) || keyboard_capturer == Some(id);

        // Only show if it has the keyboard captured.
        // Chat input uses a rectangle as its background.
        if input_focused {
            // Shallow comparison of ChatMode.
            let discrim = std::mem::discriminant;
            if discrim(&state.input.mode) != discrim(&self.client.chat_mode) {
                state.update(|s| {
                    s.input.mode = self.client.chat_mode.clone();
                });
            }

            let (color, icon) = render_chat_mode(&state.input.mode, self.imgs);
            Image::new(icon)
                .w_h(CHAT_ICON_WIDTH, CHAT_ICON_HEIGHT)
                .top_left_with_margin_on(state.ids.chat_input_bg, 2.0)
                .set(state.ids.chat_input_icon, ui);

            // Any changes to this TextEdit's width and font size must be reflected in
            // `cursor_offset_to_index` below.
            let mut text_edit = TextEdit::new(&state.input.message)
                .w(chat_box_input_width)
                .restrict_to_height(false)
                .color(color)
                .line_spacing(2.0)
                .font_size(self.fonts.universal.scale(15))
                .font_id(self.fonts.universal.conrod_id);

            if let Some(pos) = force_cursor {
                text_edit = text_edit.cursor_pos(pos);
            }

            let y = match text_edit.get_y_dimension(ui) {
                Dimension::Absolute(y) => y + 6.0,
                _ => 0.0,
            };
            Rectangle::fill([chat_size.x, y])
                .rgba(0.0, 0.0, 0.0, chat_settings.chat_opacity)
                .w(chat_size.x)
                .and(|r| {
                    if chat_in_screen_upper {
                        r.down_from(state.ids.message_box_bg, CHAT_MARGIN_THICKNESS / 2.0)
                    } else {
                        r.bottom_left_with_margins_on(ui.window, chat_pos.y, chat_pos.x)
                    }
                })
                .set(state.ids.chat_input_bg, ui);

            //border around focused chat window
            let border_color = adjust_border_opacity(color, chat_settings.chat_opacity);
            //top line
            Line::centred([0.0, 0.0], [chat_size.x, 0.0])
                .color(border_color)
                .thickness(CHAT_MARGIN_THICKNESS)
                .top_left_of(state.ids.chat_input_bg)
                .set(state.ids.chat_input_border_up, ui);
            //bottom line
            Line::centred([0.0, 0.0], [chat_size.x, 0.0])
                .color(border_color)
                .thickness(CHAT_MARGIN_THICKNESS)
                .bottom_left_of(state.ids.chat_input_bg)
                .set(state.ids.chat_input_border_down, ui);
            //left line
            Line::centred([0.0, 0.0], [0.0, y])
                .color(border_color)
                .thickness(CHAT_MARGIN_THICKNESS)
                .bottom_left_of(state.ids.chat_input_bg)
                .set(state.ids.chat_input_border_left, ui);
            //right line
            Line::centred([0.0, 0.0], [0.0, y])
                .color(border_color)
                .thickness(CHAT_MARGIN_THICKNESS)
                .bottom_right_of(state.ids.chat_input_bg)
                .set(state.ids.chat_input_border_right, ui);

            if let Some(mut input) = text_edit
                .right_from(state.ids.chat_input_icon, 1.0)
                .set(state.ids.chat_input, ui)
            {
                input.retain(|c| c != '\n');
                state.update(|s| s.input.message = input);
            }
        }

        // Message box
        Rectangle::fill([chat_size.x, chat_size.y])
            .rgba(0.0, 0.0, 0.0, chat_settings.chat_opacity)
            .and(|r| {
                if input_focused && !chat_in_screen_upper {
                    r.up_from(
                        state.ids.chat_input_border_up,
                        0.0 + CHAT_MARGIN_THICKNESS / 2.0,
                    )
                } else {
                    r.bottom_left_with_margins_on(ui.window, chat_pos.y, chat_pos.x)
                }
            })
            .crop_kids()
            .set(state.ids.message_box_bg, ui);
        if state.ids.chat_icons.len() < state.messages.len() {
            state.update(|s| {
                s.ids
                    .chat_icons
                    .resize(s.messages.len(), &mut ui.widget_id_generator())
            });
        }
        let group_members = self
            .client
            .group_members()
            .iter()
            .filter_map(|(u, r)| match r {
                Role::Member => Some(u),
                Role::Pet => None,
            })
            .collect::<HashSet<_>>();
        let show_char_name = chat_settings.chat_character_name;
        let messages = &state
            .messages
            .iter()
            .filter(|m| {
                if let Some(chat_tab) = current_chat_tab {
                    chat_tab.filter.satisfies(m, &group_members)
                } else {
                    true
                }
            })
            .map(|m| {
                let is_moderator = m
                    .uid()
                    .and_then(|uid| self.client.player_list().get(&uid).map(|i| i.is_moderator))
                    .unwrap_or(false);
                let (chat_type, text) = localize_chat_message(
                    m,
                    &self.client.lookup_msg_context(m),
                    self.localized_strings,
                    show_char_name,
                );
                (is_moderator, chat_type, text)
            })
            .collect::<Vec<_>>();
        let n_badges = messages.iter().filter(|t| t.0).count();
        if state.ids.chat_badges.len() < n_badges {
            state.update(|s| {
                s.ids
                    .chat_badges
                    .resize(n_badges, &mut ui.widget_id_generator())
            })
        }
        Rectangle::fill_with([CHAT_ICON_WIDTH, chat_size.y], color::TRANSPARENT)
            .top_left_with_margins_on(state.ids.message_box_bg, 0.0, 0.0)
            .crop_kids()
            .set(state.ids.chat_icon_align, ui);
        let (mut items, _) = List::flow_down(messages.len() + 1)
            .top_left_with_margins_on(state.ids.message_box_bg, 0.0, CHAT_ICON_WIDTH)
            .w_h(chat_size.x - CHAT_ICON_WIDTH, chat_size.y)
            .scroll_kids_vertically()
            .set(state.ids.message_box, ui);

        let mut badge_id = 0;
        while let Some(item) = items.next(ui) {
            /// Calculate the width of the group text or faction name
            fn group_width(chat_type: &ChatType<String>, ui: &Ui, font: &Font) -> Option<f64> {
                // This is a temporary solution on a best effort basis
                // This needs to be reworked in the long run
                let text = match chat_type {
                    ChatType::Group(_, desc) => desc.as_str(),
                    ChatType::Faction(_, desc) => desc.as_str(),
                    _ => return None,
                };
                let bracket_width = Text::new("() ")
                    .font_size(font.scale(15))
                    .font_id(font.conrod_id)
                    .get_w(ui)?;
                Text::new(text)
                    .font_size(font.scale(15))
                    .font_id(font.conrod_id)
                    .get_w(ui)
                    .map(|v| bracket_width + v)
            }
            // This would be easier if conrod used the v-metrics from rusttype.
            if item.i < messages.len() {
                let (is_moderator, chat_type, text) = &messages[item.i];
                let (color, icon) = render_chat_line(chat_type, self.imgs);
                // For each ChatType needing localization get/set matching pre-formatted
                // localized string. This string will be formatted with the data
                // provided in ChatType in the client/src/mod.rs
                // fn format_message called below

                let text = Text::new(text)
                    .font_size(self.fonts.universal.scale(15))
                    .font_id(self.fonts.universal.conrod_id)
                    .w(chat_size.x - CHAT_ICON_WIDTH - 1.0)
                    .wrap_by_word()
                    .color(color)
                    .line_spacing(2.0);

                // Add space between messages.
                let y = match text.get_y_dimension(ui) {
                    Dimension::Absolute(y) => y + 2.0,
                    _ => 0.0,
                };
                item.set(text.h(y), ui);

                // If the user is a moderator display a moderator icon with their alias.
                if *is_moderator {
                    let group_width =
                        group_width(chat_type, ui, &self.fonts.universal).unwrap_or(0.0);
                    Image::new(self.imgs.chat_moderator_badge)
                        .w_h(CHAT_ICON_WIDTH, CHAT_ICON_HEIGHT)
                        .top_left_with_margins_on(item.widget_id, 2.0, 7.0 + group_width)
                        .parent(state.ids.message_box_bg)
                        .set(state.ids.chat_badges[badge_id], ui);

                    badge_id += 1;
                }

                let icon_id = state.ids.chat_icons[item.i];
                Image::new(icon)
                    .w_h(CHAT_ICON_WIDTH, CHAT_ICON_HEIGHT)
                    .top_left_with_margins_on(item.widget_id, 2.0, -CHAT_ICON_WIDTH)
                    .parent(state.ids.chat_icon_align)
                    .set(icon_id, ui);
            } else {
                // Spacer at bottom of the last message so that it is not cut off.
                // Needs to be larger than the space above.
                item.set(
                    Text::new("")
                        .font_size(self.fonts.universal.scale(6))
                        .font_id(self.fonts.universal.conrod_id)
                        .w(chat_size.x),
                    ui,
                );
            };
        }

        //Chat tabs
        if ui
            .rect_of(state.ids.message_box_bg)
            .is_some_and(|r| r.is_over(ui.global_input().current.mouse.xy))
        {
            state.update(|s| s.tabs_last_hover_pulse = Some(self.pulse));
        }

        if let Some(time_since_hover) = state
            .tabs_last_hover_pulse
            .map(|t| self.pulse - t)
            .filter(|t| t <= &1.5)
        {
            let alpha = 1.0 - (time_since_hover / 1.5).powi(4);
            let shading = color::rgba(1.0, 0.82, 0.27, chat_settings.chat_opacity * alpha);

            Rectangle::fill([chat_size.x, CHAT_TAB_HEIGHT])
                .rgba(0.0, 0.0, 0.0, chat_settings.chat_opacity * alpha)
                .up_from(state.ids.message_box_bg, 0.0)
                .set(state.ids.chat_tab_align, ui);
            if ui
                .rect_of(state.ids.chat_tab_align)
                .is_some_and(|r| r.is_over(ui.global_input().current.mouse.xy))
            {
                state.update(|s| s.tabs_last_hover_pulse = Some(self.pulse));
            }

            if Button::image(if chat_settings.chat_tab_index.is_none() {
                self.imgs.selection
            } else {
                self.imgs.nothing
            })
            .top_left_with_margins_on(state.ids.chat_tab_align, 0.0, 0.0)
            .w_h(CHAT_TAB_ALL_WIDTH, CHAT_TAB_HEIGHT)
            .hover_image(self.imgs.selection_hover)
            .hover_image(self.imgs.selection_press)
            .image_color(shading)
            .label(&self.localized_strings.get_msg("hud-chat-all"))
            .label_font_size(self.fonts.cyri.scale(14))
            .label_font_id(self.fonts.cyri.conrod_id)
            .label_color(TEXT_COLOR.alpha(alpha))
            .set(state.ids.chat_tab_all, ui)
            .was_clicked()
            {
                events.push(Event::ChangeChatTab(None));
            }

            let chat_tab_width = (chat_size.x - CHAT_TAB_ALL_WIDTH) / (MAX_CHAT_TABS as f64);

            if state.ids.chat_tabs.len() < chat_tabs.len() {
                state.update(|s| {
                    s.ids
                        .chat_tabs
                        .resize(chat_tabs.len(), &mut ui.widget_id_generator())
                });
            }
            for (i, chat_tab) in chat_tabs.iter().enumerate() {
                if Button::image(if chat_settings.chat_tab_index == Some(i) {
                    self.imgs.selection
                } else {
                    self.imgs.nothing
                })
                .w_h(chat_tab_width, CHAT_TAB_HEIGHT)
                .hover_image(self.imgs.selection_hover)
                .press_image(self.imgs.selection_press)
                .image_color(shading)
                .label(chat_tab.label.as_str())
                .label_font_size(self.fonts.cyri.scale(14))
                .label_font_id(self.fonts.cyri.conrod_id)
                .label_color(TEXT_COLOR.alpha(alpha))
                .right_from(
                    if i == 0 {
                        state.ids.chat_tab_all
                    } else {
                        state.ids.chat_tabs[i - 1]
                    },
                    0.0,
                )
                .set(state.ids.chat_tabs[i], ui)
                .was_clicked()
                {
                    events.push(Event::ChangeChatTab(Some(i)));
                }

                if ui
                    .widget_input(state.ids.chat_tabs[i])
                    .mouse()
                    .is_some_and(|m| m.is_over())
                {
                    Rectangle::fill([120.0, 20.0])
                        .rgba(0.0, 0.0, 0.0, 0.9)
                        .top_left_with_margins_on(state.ids.chat_tabs[i], -20.0, 5.0)
                        .parent(id)
                        .set(state.ids.chat_tab_tooltip_bg, ui);

                    Text::new(
                        &self
                            .localized_strings
                            .get_msg("hud-chat-chat_tab_hover_tooltip"),
                    )
                    .mid_top_with_margin_on(state.ids.chat_tab_tooltip_bg, 3.0)
                    .font_size(self.fonts.cyri.scale(10))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.chat_tab_tooltip_text, ui);
                }

                if ui
                    .widget_input(state.ids.chat_tabs[i])
                    .clicks()
                    .right()
                    .next()
                    .is_some()
                {
                    events.push(Event::ShowChatTabSettings(i));
                }
            }
        }

        // Chat Arrow
        // Check if already at bottom.
        if !Self::scrolled_to_bottom(state, ui)
            && Button::image(self.imgs.chat_arrow)
                .w_h(20.0, 20.0)
                .hover_image(self.imgs.chat_arrow_mo)
                .press_image(self.imgs.chat_arrow_press)
                .top_right_with_margins_on(state.ids.message_box_bg, 0.0, -22.0)
                .parent(id)
                .set(state.ids.chat_arrow, ui)
                .was_clicked()
        {
            ui.scroll_widget(state.ids.message_box, [0.0, f64::MAX]);
        }

        // We've started a new tab completion. Populate tab completion suggestions.
        if request_tab_completions {
            events.push(Event::TabCompletionStart(state.input.message.to_string()));
        // If the chat widget is focused, return a focus event to pass the focus
        // to the input box.
        } else if keyboard_capturer == Some(id) {
            events.push(Event::Focus(state.ids.chat_input));
        }
        // If either Return or Enter is pressed and the input box is not empty, send the current
        // message.
        else if ui
            .widget_input(state.ids.chat_input)
            .presses()
            .key()
            .any(|key_press| {
                let has_message = !state.input.message.is_empty();
                let pressed = matches!(key_press.key, Key::Return | Key::NumPadEnter);
                if pressed {
                    // If chat was hidden, scroll to bottom the next time it is opened
                    state.update(|s| s.scroll_next |= force_chat);
                    events.push(Event::DisableForceChat);
                }
                has_message && pressed
            })
        {
            let msg = state.input.message.clone();
            state.update(|s| {
                s.input.message.clear();
                // Update the history
                // Don't add if this is identical to the last message in the history
                s.history_pos = 0;
                if s.history.front() != Some(&msg) {
                    s.history.push_front(msg.clone());
                    s.history.truncate(self.history_max);
                }
            });
            if let Some(msg) = msg.strip_prefix(chat_settings.chat_cmd_prefix) {
                match parse_cmd(msg) {
                    Ok((name, args)) => events.push(Event::SendCommand(name, args)),
                    // TODO: Localise
                    Err(err) => self
                        .new_messages
                        .push_back(ChatType::CommandError.into_plain_msg(err)),
                }
            } else {
                events.push(Event::SendMessage(msg));
            }
        }

        Rectangle::fill_with([chat_size.x, chat_size.y], color::TRANSPARENT)
            .and(|r| {
                if input_focused {
                    r.up_from(state.ids.chat_input_border_up, CHAT_MARGIN_THICKNESS / 2.0)
                } else {
                    r.bottom_left_with_margins_on(ui.window, chat_pos.y, chat_pos.x)
                }
            })
            .set(state.ids.draggable_area, ui);
        events
    }
}

fn do_tab_completion(cursor: usize, input: &str, word: &str) -> (String, usize) {
    let mut pre_ws = None;
    let mut post_ws = None;
    let mut in_quotation = false;
    for (char_i, (byte_i, c)) in input.char_indices().enumerate() {
        if c == '"' {
            in_quotation = !in_quotation;
        } else if !in_quotation && c.is_whitespace() && c != '\t' {
            if char_i < cursor {
                pre_ws = Some(byte_i);
            } else {
                post_ws = Some(byte_i);
                break;
            }
        }
    }

    match (pre_ws, post_ws) {
        (None, None) => (word.to_string(), word.chars().count()),
        (None, Some(i)) => (
            format!("{}{}", word, input.split_at(i).1),
            word.chars().count(),
        ),
        (Some(i), None) => {
            let l_split = input.split_at(i).0;
            let completed = format!("{} {}", l_split, word);
            (
                completed,
                l_split.chars().count() + 1 + word.chars().count(),
            )
        },
        (Some(i), Some(j)) => {
            let l_split = input.split_at(i).0;
            let r_split = input.split_at(j).1;
            let completed = format!("{} {}{}", l_split, word, r_split);
            (
                completed,
                l_split.chars().count() + 1 + word.chars().count(),
            )
        },
    }
}

fn cursor_offset_to_index(
    offset: usize,
    text: &str,
    ui: &Ui,
    fonts: &Fonts,
    input_width: f64,
) -> Option<Index> {
    // This moves the cursor to the given offset. Conrod is a pain.
    //
    // Width and font must match that of the chat TextEdit
    let font = ui.fonts.get(fonts.universal.conrod_id)?;
    let font_size = fonts.universal.scale(15);
    let infos = text::line::infos(text, font, font_size).wrap_by_whitespace(input_width);

    cursor::index_before_char(infos, offset)
}

/// Get the color and icon for a client's ChatMode.
fn render_chat_mode(chat_mode: &ChatMode, imgs: &Imgs) -> (Color, conrod_core::image::Id) {
    match chat_mode {
        ChatMode::World => (WORLD_COLOR, imgs.chat_world_small),
        ChatMode::Say => (SAY_COLOR, imgs.chat_say_small),
        ChatMode::Region => (REGION_COLOR, imgs.chat_region_small),
        ChatMode::Faction(_) => (FACTION_COLOR, imgs.chat_faction_small),
        ChatMode::Group => (GROUP_COLOR, imgs.chat_group_small),
        ChatMode::Tell(_) => (TELL_COLOR, imgs.chat_tell_small),
    }
}

/// Get the color and icon for the current line in the chat box
fn render_chat_line(chat_type: &ChatType<String>, imgs: &Imgs) -> (Color, conrod_core::image::Id) {
    match chat_type {
        ChatType::Online(_) => (ONLINE_COLOR, imgs.chat_online_small),
        ChatType::Offline(_) => (OFFLINE_COLOR, imgs.chat_offline_small),
        ChatType::CommandError => (ERROR_COLOR, imgs.chat_command_error_small),
        ChatType::CommandInfo => (INFO_COLOR, imgs.chat_command_info_small),
        ChatType::GroupMeta(_) => (GROUP_COLOR, imgs.chat_group_small),
        ChatType::FactionMeta(_) => (FACTION_COLOR, imgs.chat_faction_small),
        ChatType::Kill(_, _) => (KILL_COLOR, imgs.chat_kill_small),
        ChatType::Tell(_from, _to) => (TELL_COLOR, imgs.chat_tell_small),
        ChatType::Say(_uid) => (SAY_COLOR, imgs.chat_say_small),
        ChatType::Group(_uid, _s) => (GROUP_COLOR, imgs.chat_group_small),
        ChatType::Faction(_uid, _s) => (FACTION_COLOR, imgs.chat_faction_small),
        ChatType::Region(_uid) => (REGION_COLOR, imgs.chat_region_small),
        ChatType::World(_uid) => (WORLD_COLOR, imgs.chat_world_small),
        ChatType::Npc(_uid) => panic!("NPCs can't talk!"), // Should be filtered by hud/mod.rs
        ChatType::NpcSay(_uid) => (SAY_COLOR, imgs.chat_say_small),
        ChatType::NpcTell(_from, _to) => (TELL_COLOR, imgs.chat_tell_small),
        ChatType::Meta => (INFO_COLOR, imgs.chat_command_info_small),
    }
}

fn parse_cmd(msg: &str) -> Result<(String, Vec<String>), String> {
    use chumsky::prelude::*;

    let escape = just::<_, _, Simple<char>>('\\').ignore_then(
        just('\\')
            .or(just('/'))
            .or(just('"'))
            .or(just('b').to('\x08'))
            .or(just('f').to('\x0C'))
            .or(just('n').to('\n'))
            .or(just('r').to('\r'))
            .or(just('t').to('\t')),
    );

    let string = just('"')
        .ignore_then(filter(|c| *c != '\\' && *c != '"').or(escape).repeated())
        .then_ignore(just('"'))
        .labelled("quoted argument");

    let arg = string
        .or(filter(|c: &char| !c.is_whitespace() && *c != '"')
            .repeated()
            .at_least(1)
            .labelled("argument"))
        .collect::<String>();

    let cmd = text::ident()
        .then(arg.padded().repeated())
        .then_ignore(end());

    cmd.parse(msg).map_err(|errs| {
        errs.into_iter()
            .map(|err| err.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    })
}

/// Change the chat mode if we have a `ServerChatCommand` that corresponds to a
/// chat region (i.e. World, Region, Say, etc.).
fn change_chat_mode(
    message: String,
    state: &mut conrod_core::widget::State<State>,
    events: &mut Vec<Event>,
    chat_settings: &ChatSettings,
) {
    if let Some(msg) = message.strip_prefix(chat_settings.chat_cmd_prefix) {
        match parse_cmd(msg.trim()) {
            Ok((name, args)) => {
                #[expect(clippy::collapsible_match)]
                if let Ok(command) = name.parse::<ServerChatCommand>() {
                    match command {
                        ServerChatCommand::Group
                        | ServerChatCommand::Say
                        | ServerChatCommand::Faction
                        | ServerChatCommand::Region
                        | ServerChatCommand::World => {
                            // Only remove the command if there is no message
                            if args.is_empty() {
                                // We found a match to a command so clear the input
                                // message
                                state.update(|s| s.input.message.clear());
                                events.push(Event::SendCommand(name, args))
                            }
                        },
                        // TODO: Add support for Whispers (might need to adjust widget
                        // for this.)
                        _ => (),
                    }
                }
            },
            // Do nothing because we are just completing the Chat Mode
            Err(_) => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cmds() {
        let expected: Result<(String, Vec<String>), String> = Ok(("help".to_string(), vec![]));
        assert_eq!(parse_cmd(r"help"), expected);

        let expected: Result<(String, Vec<String>), String> = Ok(("say".to_string(), vec![
            "foo".to_string(),
            "bar".to_string(),
        ]));
        assert_eq!(parse_cmd(r"say foo bar"), expected);
        assert_eq!(parse_cmd(r#"say "foo" "bar""#), expected);

        let expected: Result<(String, Vec<String>), String> =
            Ok(("say".to_string(), vec!["Hello World".to_string()]));
        assert_eq!(parse_cmd(r#"say "Hello World""#), expected);

        // Note: \n in the expected gets expanded by rust to a newline character, that's
        // why we must not use a raw string in the expected
        let expected: Result<(String, Vec<String>), String> =
            Ok(("say".to_string(), vec!["Hello\nWorld".to_string()]));
        assert_eq!(parse_cmd(r#"say "Hello\nWorld""#), expected);
    }
}
