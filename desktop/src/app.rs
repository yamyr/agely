use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;

use gpui::{
    AccessibleAction, AnyElement, App, Context, Entity, FocusHandle, FontWeight, Role,
    ScrollHandle, SharedString, Window, actions, div, point, prelude::*, px, relative,
};
use time::{Duration, OffsetDateTime, Time};

use crate::agent::{AgentState, AgentStatus, StatusState, TurnRequest};
use crate::i18n::{I18n, parse_dollars, save_locale};
use crate::input::TextInput;
use crate::live::Conversation;
use crate::model::{Action, CHAPTER_NAMES, CHAPTER_STEPS, DemoState, Notice, NoticeArg, Seed};
use crate::theme::theme;
use crate::widgets::{ControlFocus, column, copy, heading, muted, notice, row};

actions!(agely, [Quit, Tab, PreviousTab, DismissLanguages]);

const CONVERSATIONS: [&str; 3] = ["interview", "brief", "debrief"];

#[derive(Clone)]
pub(crate) enum UiAction {
    Domain(Action),
    SavePrice,
    ConfirmBrief,
    SendProposal,
    RenameSkill(String),
    Recheck,
    ToggleLive,
    StartLive(usize),
    ReplyLive(usize, String),
    SendLive(usize),
    FinishLive(usize),
    Scripted(usize),
    ToggleLanguages,
    ChooseLanguage(String),
    ConfirmLanguage,
    KeepLanguage,
}

pub struct Agely {
    pub(crate) state: DemoState,
    pub(crate) i18n: I18n,
    pub(crate) fields: BTreeMap<String, Entity<TextInput>>,
    pub(crate) live: [Conversation; 3],
    pub(crate) live_on: bool,
    pub(crate) status: Option<AgentStatus>,
    pub(crate) checking: bool,
    pub(crate) now: OffsetDateTime,
    agent: Arc<AgentState>,
    generation: u64,
    next_request: u64,
    heading_focus: FocusHandle,
    root_focus: FocusHandle,
    language_menu_focus: FocusHandle,
    language_cancel_focus: FocusHandle,
    scroll: ScrollHandle,
    sidebar_scroll: ScrollHandle,
    language_scroll: ScrollHandle,
    controls: RefCell<BTreeMap<SharedString, ControlFocus>>,
    languages_open: bool,
    language_confirmation: Option<String>,
}

impl Agely {
    pub fn new(i18n: I18n, seed: Seed, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut app = Self::create(i18n, seed, window, cx);
        app.check_status(false, cx);
        app
    }

    fn create(i18n: I18n, seed: Seed, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let root_focus = cx.focus_handle();
        window.focus(&root_focus, cx);
        Self {
            state: DemoState::new(seed),
            i18n,
            fields: BTreeMap::new(),
            live: std::array::from_fn(|_| Conversation::default()),
            live_on: true,
            status: None,
            checking: false,
            now: OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc()),
            agent: Arc::new(AgentState::new()),
            generation: 1,
            next_request: 1,
            heading_focus: cx.focus_handle(),
            root_focus,
            language_menu_focus: cx.focus_handle().tab_stop(true),
            language_cancel_focus: cx.focus_handle().tab_stop(true),
            scroll: ScrollHandle::default(),
            sidebar_scroll: ScrollHandle::default(),
            language_scroll: ScrollHandle::default(),
            controls: RefCell::default(),
            languages_open: false,
            language_confirmation: None,
        }
    }

    pub(crate) fn t(&self, source: &str) -> String {
        self.i18n.t(source, &[])
    }

    pub(crate) fn message(&self, source: &str, values: &[String]) -> String {
        self.i18n.t(source, values)
    }

    pub(crate) fn range(&self, range: crate::model::PriceRange) -> String {
        self.message(
            "{0}–{1}",
            &[self.i18n.money(range.min), self.i18n.money(range.max)],
        )
    }

    pub(crate) fn notice_text(&self, value: &Notice) -> String {
        let args = value
            .args
            .iter()
            .map(|arg| match arg {
                NoticeArg::Text(text) => text.clone(),
                NoticeArg::Number(number) => self.i18n.number(*number),
                NoticeArg::Money(amount) => self.i18n.money(*amount),
                NoticeArg::Range(range) => self.range(*range),
            })
            .collect::<Vec<_>>();
        self.message(&value.source, &args)
    }

    pub(crate) fn button(
        &self,
        id: impl Into<SharedString>,
        label: String,
        action: UiAction,
        enabled: bool,
        primary: bool,
        cx: &Context<Self>,
    ) -> AnyElement {
        let (button, focus) = self.button_control(id, label, action, enabled, primary, cx);
        focus.reveal(button, &self.scroll)
    }

    fn chrome_button(
        &self,
        id: impl Into<SharedString>,
        label: String,
        action: UiAction,
        enabled: bool,
        primary: bool,
        cx: &Context<Self>,
    ) -> AnyElement {
        self.button_control(id, label, action, enabled, primary, cx)
            .0
    }

    fn button_control(
        &self,
        id: impl Into<SharedString>,
        label: String,
        action: UiAction,
        enabled: bool,
        primary: bool,
        cx: &Context<Self>,
    ) -> (AnyElement, ControlFocus) {
        let id: SharedString = id.into();
        let focus = self
            .controls
            .borrow_mut()
            .entry(id.clone())
            .or_insert_with(|| {
                ControlFocus::new(match id.as_ref() {
                    "language-menu" => self.language_menu_focus.clone(),
                    "keep-language" => self.language_cancel_focus.clone(),
                    _ => cx.focus_handle().tab_stop(true),
                })
            })
            .clone();
        let clicked = action.clone();
        let accessible = cx.entity().downgrade();
        let debug_id = id.clone();
        let mut button = div()
            .id(id.clone())
            .accessibility_id(id.clone())
            .debug_selector(move || debug_id.to_string())
            .role(if enabled { Role::Button } else { Role::Label })
            .aria_label(SharedString::from(label.clone()))
            .min_h(px(theme().control))
            .min_w_0()
            .max_w_full()
            .px(px(theme().space_large))
            .py(px(theme().space_small))
            .rounded(px(theme().radius))
            .border_2()
            .border_color(if primary {
                theme().accent
            } else {
                theme().border
            })
            .bg(if primary {
                theme().accent
            } else {
                theme().surface
            })
            .text_color(if primary {
                theme().surface
            } else {
                theme().text
            })
            .font_weight(FontWeight::SEMIBOLD)
            .line_height(relative(1.4))
            .flex()
            .items_center()
            .child(div().min_w_0().whitespace_normal().child(label));
        if enabled {
            button = button
                .track_focus(&focus.handle)
                .tab_stop(true)
                .cursor_pointer()
                .focus_visible(|style| {
                    style
                        .border(px(theme().focus_width))
                        .border_color(theme().text)
                })
                .hover(|style| style.border_color(theme().accent))
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.handle(clicked.clone(), window, cx);
                }))
                .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                    let _ = accessible.update(cx, |this, cx| {
                        this.handle(action.clone(), window, cx);
                    });
                });
        } else {
            button = button.opacity(0.55);
        }
        (button.into_any_element(), focus)
    }

    pub(crate) fn decision(
        &self,
        id: &'static str,
        label: &str,
        action: Action,
        cx: &Context<Self>,
    ) -> AnyElement {
        self.button(id, self.t(label), UiAction::Domain(action), true, true, cx)
    }

    pub(crate) fn field(
        &mut self,
        id: &str,
        label: String,
        initial: String,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let field = if let Some(field) = self.fields.get(id) {
            let field = field.clone();
            field.update(cx, |field, cx| field.set_label(label.clone(), cx));
            field
        } else {
            let mut previous_value = initial.clone();
            let field = cx.new(|cx| TextInput::new(initial, label.clone(), cx));
            let key = id.to_owned();
            cx.observe(&field, move |this, field, cx| {
                let value = field.read(cx).value();
                if value == previous_value {
                    return;
                }
                previous_value = value.to_owned();
                if key == "brief-problem" || key.starts_with("budget-") {
                    this.state.dispatch(Action::EditBrief {
                        problem: this.field_value("brief-problem", cx),
                        units: this.state.brief.units,
                        budget: crate::model::PriceRange {
                            min: this
                                .amount("budget-min", cx)
                                .unwrap_or(this.state.brief.budget.min),
                            max: this
                                .amount("budget-max", cx)
                                .unwrap_or(this.state.brief.budget.max),
                        },
                    });
                } else if key == "proposal-price" {
                    if let Some(price_per_unit) = this.amount("proposal-price", cx) {
                        this.state
                            .dispatch(Action::SetProposalPrice { price_per_unit });
                    } else {
                        this.state.proposal.sent = false;
                        this.state.notice = None;
                    }
                }
                cx.notify();
            })
            .detach();
            self.fields.insert(id.to_owned(), field.clone());
            field
        };
        let handle = field.read(cx).focus_handle.clone();
        let focus = self
            .controls
            .borrow_mut()
            .entry(SharedString::from(format!("field-{id}")))
            .and_modify(|focus| {
                if focus.handle != handle {
                    *focus = ControlFocus::new(handle.clone());
                }
            })
            .or_insert_with(|| ControlFocus::new(handle))
            .clone();
        column(vec![
            copy(SharedString::from(format!("{id}-label")), label),
            focus.reveal(field.into_any_element(), &self.scroll),
        ])
    }

    fn field_value(&self, id: &str, cx: &App) -> String {
        self.fields
            .get(id)
            .map(|field| field.read(cx).value().to_owned())
            .unwrap_or_default()
    }

    fn amount(&self, id: &str, cx: &App) -> Option<i64> {
        parse_dollars(&self.field_value(id, cx))
    }

    pub(crate) fn is_live(&self, index: usize) -> bool {
        self.live_on
            && !self.checking
            && !self.live[index].scripted
            && self
                .status
                .as_ref()
                .is_some_and(|status| status.state == StatusState::Ready)
    }

    fn can_advance(&self) -> bool {
        if let Some(index) = self.state.conversation_index()
            && self.is_live(index)
        {
            let conversation = &self.live[index];
            return conversation.ended
                && conversation.pending.is_none()
                && conversation.error.is_none();
        }
        self.state.can_advance()
    }

    fn advance_hint(&self) -> String {
        if self.can_advance() {
            return String::new();
        }
        if let Some(index) = self.state.conversation_index()
            && self.is_live(index)
        {
            let conversation = &self.live[index];
            return self.t(if conversation.pending.is_some() {
                "The agent is thinking…"
            } else if conversation.lines.is_empty() {
                "Start the conversation"
            } else {
                "Finish conversation"
            });
        }
        self.state
            .blocked_reason()
            .map(|reason| self.t(reason))
            .unwrap_or_default()
    }

    fn remember_scripted(&mut self) {
        for index in 0..3 {
            if self.state.shown[index] > 0 && self.live[index].lines.is_empty() {
                self.live[index].fallback();
            }
        }
    }

    fn check_status(&mut self, recheck: bool, cx: &mut Context<Self>) {
        if self.checking {
            return;
        }
        self.checking = true;
        let agent = Arc::clone(&self.agent);
        let task = cx
            .background_executor()
            .spawn(async move { agent.status(recheck) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.checking = false;
                this.status = Some(result.unwrap_or_else(|detail| AgentStatus {
                    state: StatusState::Error,
                    model: None,
                    detail,
                    fix: Some("Select Check again.".into()),
                }));
                this.remember_scripted();
                cx.notify();
            });
        })
        .detach();
    }

    fn start_live(&mut self, index: usize, reply: Option<String>, cx: &mut Context<Self>) {
        if !self.is_live(index) {
            return;
        }
        let request_id = self.next_request;
        self.next_request += 1;
        let requested = match reply {
            Some(text) => self.live[index].reply(&text, request_id),
            None => self.live[index].request(request_id),
        };
        if !requested {
            return;
        }
        let Some(brief) = self.i18n.live_brief(CONVERSATIONS[index]) else {
            self.live[index].fail(
                request_id,
                self.t("The agent could not reply. Try again, or use the scripted conversation."),
            );
            return;
        };
        let request = TurnRequest {
            brief,
            transcript: self.live[index].lines.clone(),
        };
        let generation = self.generation;
        let agent = Arc::clone(&self.agent);
        let task = cx
            .background_executor()
            .spawn(async move { agent.turn(request) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                match result {
                    Ok(turn) => this.live[index].receive(request_id, turn),
                    Err(error) => this.live[index].fail(request_id, error.message),
                }
                if this.live[index].ended {
                    this.state.shown[index] = this.state.conversation_turns(index).len();
                }
                if this.state.conversation_index() == Some(index) {
                    this.scroll.scroll_to_bottom();
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn new_run(&mut self) {
        self.generation += 1;
        self.live = std::array::from_fn(|_| Conversation::default());
        self.fields.clear();
        self.now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    }

    fn focus_heading(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.scroll.set_offset(point(px(0.0), px(0.0)));
        window.focus(&self.heading_focus, cx);
    }

    fn invalid_amount(&mut self) {
        self.state.notice = Some(Notice::error(
            "Please use whole dollar amounts, like 60 or 90.",
        ));
    }

    pub(crate) fn handle(&mut self, action: UiAction, window: &mut Window, cx: &mut Context<Self>) {
        let previous_step = self.state.step;
        let mut reset = false;
        match action {
            UiAction::Domain(action) => {
                let reveal = matches!(action, Action::ShowNextTurn);
                if matches!(action, Action::Next) && !self.can_advance() {
                    return;
                }
                if matches!(action, Action::ReplyToAgent { .. }) {
                    self.fields.clear();
                }
                if matches!(action, Action::RedoInterview) {
                    self.live[0] = Conversation::default();
                    self.fields.clear();
                }
                if matches!(action, Action::Restart | Action::JumpTo { .. }) {
                    self.new_run();
                    reset = true;
                }
                self.state.dispatch(action);
                self.remember_scripted();
                if reveal {
                    self.scroll.scroll_to_bottom();
                }
            }
            UiAction::SavePrice => {
                match (self.amount("price-min", cx), self.amount("price-max", cx)) {
                    (Some(min), Some(max)) => self.state.dispatch(Action::SetPrice {
                        range: crate::model::PriceRange { min, max },
                    }),
                    _ => self.invalid_amount(),
                }
            }
            UiAction::ConfirmBrief => {
                match (self.amount("budget-min", cx), self.amount("budget-max", cx)) {
                    (Some(min), Some(max)) => {
                        self.state.dispatch(Action::EditBrief {
                            problem: self.field_value("brief-problem", cx),
                            units: self.state.brief.units,
                            budget: crate::model::PriceRange { min, max },
                        });
                        self.state.dispatch(Action::ConfirmBrief);
                    }
                    _ => self.invalid_amount(),
                }
            }
            UiAction::SendProposal => {
                if let Some(price_per_unit) = self.amount("proposal-price", cx) {
                    self.state
                        .dispatch(Action::SetProposalPrice { price_per_unit });
                    self.state.dispatch(Action::SendProposal);
                } else {
                    self.invalid_amount();
                }
            }
            UiAction::RenameSkill(id) => {
                self.state.dispatch(Action::EditCompetency {
                    label: self.field_value(&format!("skill-{id}"), cx),
                    id,
                });
            }
            UiAction::Recheck => self.check_status(true, cx),
            UiAction::ToggleLive => {
                self.live_on = !self.live_on;
                self.remember_scripted();
            }
            UiAction::StartLive(index) => self.start_live(index, None, cx),
            UiAction::ReplyLive(index, text) => self.start_live(index, Some(text), cx),
            UiAction::SendLive(index) => {
                let id = format!("live-reply-{index}");
                let text = self.field_value(&id, cx);
                self.start_live(index, Some(text), cx);
                if self.live[index].pending.is_some()
                    && let Some(field) = self.fields.get(&id)
                {
                    field.update(cx, |field, cx| field.set_value("", cx));
                }
            }
            UiAction::FinishLive(index) => {
                self.live[index].finish();
                if self.live[index].ended {
                    self.state.shown[index] = self.state.conversation_turns(index).len();
                }
            }
            UiAction::Scripted(index) => self.live[index].fallback(),
            UiAction::ToggleLanguages => {
                self.languages_open = !self.languages_open;
                window.focus(&self.language_menu_focus, cx);
            }
            UiAction::ChooseLanguage(locale) => {
                self.languages_open = false;
                if locale != self.i18n.locale() {
                    self.language_confirmation = Some(locale);
                    window.focus(&self.language_cancel_focus, cx);
                } else {
                    window.focus(&self.language_menu_focus, cx);
                }
            }
            UiAction::ConfirmLanguage => {
                if let Some(locale) = self.language_confirmation.take() {
                    let i18n = self.i18n.with_locale(&locale);
                    match i18n.localized_seed() {
                        Ok(seed) => {
                            self.new_run();
                            self.i18n = i18n;
                            self.state = DemoState::new(seed);
                            reset = true;
                            if save_locale(&locale).is_err() {
                                self.state.notice = Some(Notice::error(
                                    "Your changes could not be saved. Please try again.",
                                ));
                            }
                        }
                        Err(error) => self.state.notice = Some(Notice::error(error.to_string())),
                    }
                }
            }
            UiAction::KeepLanguage => {
                self.language_confirmation = None;
                window.focus(&self.language_menu_focus, cx);
            }
        }
        if previous_step != self.state.step || reset {
            self.fields.clear();
            self.focus_heading(window, cx);
        }
        cx.notify();
    }

    pub(crate) fn session_time(&self) -> String {
        let date = self.now.checked_add(Duration::days(2)).unwrap_or(self.now);
        let local = date.replace_time(Time::from_hms(10, 0, 0).expect("valid demo time"));
        self.i18n.session_time(local)
    }

    fn language_controls(&self, cx: &mut Context<Self>) -> AnyElement {
        if let Some(locale) = &self.language_confirmation {
            let label = self
                .i18n
                .locales()
                .iter()
                .find(|entry| &entry.id == locale)
                .map(|entry| entry.label.clone())
                .unwrap_or_else(|| locale.clone());
            return div().id("language-confirmation").role(Role::Dialog)
                .aria_label(self.t("Change language")).child(column(vec![
                    heading("language-confirmation-title", self.t("Change language"), 2),
                    copy("language-target", label),
                    copy("language-restart-notice", self.t("Changing language restarts the walkthrough. Your edits and conversations will be cleared.")),
                    row(vec![
                        self.chrome_button("confirm-language", self.t("Restart in this language"), UiAction::ConfirmLanguage, true, true, cx),
                        self.chrome_button("keep-language", self.t("Keep current language"), UiAction::KeepLanguage, true, false, cx),
                    ]),
                ])).into_any_element();
        }
        if self.languages_open {
            let buttons = self
                .i18n
                .locales()
                .iter()
                .map(|locale| {
                    let (button, focus) = self.button_control(
                        SharedString::from(format!("locale-{}", locale.id)),
                        locale.label.clone(),
                        UiAction::ChooseLanguage(locale.id.clone()),
                        true,
                        false,
                        cx,
                    );
                    focus.reveal(button, &self.language_scroll)
                })
                .collect();
            return div()
                .id("language-options")
                .max_h(px(theme().control * 3.0 + theme().space * 2.0))
                .overflow_y_scroll()
                .track_scroll(&self.language_scroll)
                .py(px(theme().space_small))
                .child(row(buttons))
                .into_any_element();
        }
        div().into_any_element()
    }

    fn sidebar(&self, cx: &Context<Self>) -> AnyElement {
        let current = CHAPTER_STEPS
            .iter()
            .rposition(|step| *step <= self.state.step);
        let chapters = CHAPTER_NAMES
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let (button, focus) = self.button_control(
                    SharedString::from(format!("chapter-{index}")),
                    self.t(name),
                    UiAction::Domain(Action::JumpTo { chapter: index }),
                    true,
                    current == Some(index),
                    cx,
                );
                focus.reveal(button, &self.sidebar_scroll)
            })
            .collect::<Vec<_>>();
        div()
            .id("chapters")
            .role(Role::Navigation)
            .aria_label(self.t("Chapters"))
            .w(px(220.0))
            .min_h_0()
            .flex_shrink_0()
            .overflow_y_scroll()
            .track_scroll(&self.sidebar_scroll)
            .p(px(theme().space))
            .bg(theme().muted_surface)
            .child(column(chapters))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests;

impl Render for Agely {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Conditional controls can be absent when they lose focus.
        for control in self.controls.borrow().values() {
            control.reset_unfocused(window);
        }
        if self.language_confirmation.is_some() {
            return div()
                .id("agely")
                .role(Role::Application)
                .aria_label("Agely")
                .track_focus(&self.root_focus)
                .key_context("Agely")
                .on_action(cx.listener(|_, _: &Tab, window, cx| window.focus_next(cx)))
                .on_action(cx.listener(|_, _: &PreviousTab, window, cx| window.focus_prev(cx)))
                .on_action(cx.listener(|this, _: &DismissLanguages, window, cx| {
                    this.handle(UiAction::KeepLanguage, window, cx);
                }))
                .size_full()
                .bg(theme().background)
                .text_color(theme().text)
                .font_family(".AppleSystemUIFont")
                .text_size(px(theme().body))
                .line_height(relative(1.5))
                .p(px(theme().space_large))
                .child(self.language_controls(cx));
        }
        let language_label = self
            .i18n
            .locales()
            .iter()
            .find(|entry| entry.id == self.i18n.locale())
            .map(|entry| entry.label.clone())
            .unwrap_or_else(|| "English".into());
        let mut content = vec![
            div()
                .id("screen-title")
                .role(Role::Heading)
                .aria_level(2)
                .aria_label(SharedString::from(self.screen_title()))
                .track_focus(&self.heading_focus)
                .font_family("Fraunces")
                .font_weight(FontWeight::SEMIBOLD)
                .text_size(px(theme().title))
                .line_height(relative(1.25))
                .child(self.screen_title())
                .into_any_element(),
            self.screen(cx),
        ];
        if self.state.step > 0 {
            let persona = self.t(match self.state.step {
                4..=7 => "Now: Dev, the buyer",
                9 => "Now: Margaret and Dev",
                _ => "Now: Margaret, the expert",
            });
            content.insert(0, notice("current-persona", persona, false));
        }
        if let Some(value) = &self.state.notice {
            content.push(notice(
                "decision-notice",
                self.notice_text(value),
                value.kind == crate::model::NoticeKind::Error,
            ));
        }
        let navigation = row(vec![
            self.chrome_button(
                "back",
                self.t("Back"),
                UiAction::Domain(Action::Back),
                self.state.step > 0,
                false,
                cx,
            ),
            self.chrome_button(
                "restart",
                self.t("Restart"),
                UiAction::Domain(Action::Restart),
                true,
                false,
                cx,
            ),
            self.chrome_button(
                "next",
                self.t("Next"),
                UiAction::Domain(Action::Next),
                self.can_advance(),
                true,
                cx,
            ),
        ]);
        let progress = self.message(
            "Step {0} of {1}{2}",
            &[
                self.i18n.number(self.state.step as i64 + 1),
                self.i18n.number(13),
                CHAPTER_STEPS
                    .iter()
                    .rposition(|step| *step <= self.state.step)
                    .map(|index| format!(" · {}", self.t(CHAPTER_NAMES[index])))
                    .unwrap_or_default(),
            ],
        );
        div().id("agely").role(Role::Application).aria_label("Agely")
            .track_focus(&self.root_focus).key_context("Agely")
            .on_action(cx.listener(|_, _: &Tab, window, cx| window.focus_next(cx)))
            .on_action(cx.listener(|_, _: &PreviousTab, window, cx| window.focus_prev(cx)))
            .on_action(cx.listener(|this, _: &DismissLanguages, window, cx| {
                if this.languages_open {
                    this.languages_open = false;
                    window.focus(&this.language_menu_focus, cx);
                    cx.notify();
                }
            }))
            .size_full().flex().flex_col().bg(theme().background)
            .text_color(theme().text).font_family(".AppleSystemUIFont")
            .text_size(px(theme().body)).line_height(relative(1.5))
            .child(div().flex().justify_between().items_center()
                .px(px(theme().space_large)).py(px(theme().space))
                .border_b_1().border_color(theme().border)
                .child(heading("brand", "Agely", 2))
                .child(self.chrome_button("language-menu", format!("{} · {language_label}", self.t("Language")),
                    UiAction::ToggleLanguages, true, false, cx)))
            .child(div().px(px(theme().space_large)).child(self.language_controls(cx)))
            .when(self.i18n.locale() != "en", |root| root.child(
                div().px(px(theme().space_large)).child(muted("draft-translation-notice",
                    self.t("Translations are machine-generated drafts. Missing translations appear in English. Please review before relying on them.")))))
            .child(div().flex().items_center().justify_between()
                .px(px(theme().space_large)).py(px(theme().space_small))
                .child(copy("demo-disclosure", self.t("Demo — scripted data")))
                .child(copy("progress", progress)))
            .child(div().flex().flex_1().min_h_0()
                .child(self.sidebar(cx))
                .child(div().id("walkthrough-content").flex_1().min_w_0()
                    .overflow_y_scroll().track_scroll(&self.scroll)
                    .p(px(theme().space_large)).child(column(content))))
            .child(div().flex().flex_col().gap(px(theme().space_small))
                .border_t_1().border_color(theme().border)
                .px(px(theme().space_large)).py(px(theme().space))
                .child(row(vec![
                    navigation,
                    muted("next-requirement", self.advance_hint()),
                ]))
                .child(muted("version", format!("Agely v{}", crate::VERSION))))
    }
}
