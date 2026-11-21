//! The thirteen native walkthrough screens use the same authored story and decisions as the web.

use std::slice::from_ref;

use gpui::{AnyElement, Context, SharedString};

use crate::agent::{Speaker as AgentSpeaker, StatusState};
use crate::app::{Agely, UiAction};
use crate::i18n::parse_dollars;
use crate::model::{
    Action, CompetencySource, MatchResult, NotMatchedReason, Proposal, SessionStatus, Speaker,
    WorkflowStatus, validate_proposal,
};
use crate::widgets::{card, column, copy, heading, muted, notice, row};

impl Agely {
    pub(crate) fn screen_title(&self) -> String {
        match self.state.step {
            0 => self.t("Experience shouldn't retire."),
            1 => self.t("Margaret talks to the onboarding agent"),
            2 => self.t(if self.state.margaret.published {
                "Margaret's profile is live"
            } else {
                "Margaret's draft profile"
            }),
            3 => self.t("What should Margaret charge?"),
            4 => self.t("Dev describes his problem"),
            5 => self.t("Dev's brief"),
            6 => match self.state.current_match() {
                MatchResult::Match { expert, .. } => {
                    self.message("Agely matched Dev with {0}", &[expert.name])
                }
                MatchResult::None { .. } => self.t("No match yet"),
            },
            7 => self.t(
                if matches!(self.state.current_match(), MatchResult::Match { .. }) {
                    "Dev proposes a session to Margaret"
                } else {
                    "Nobody to propose to yet"
                },
            ),
            8 => self.t("Dev would like a session with Margaret"),
            9 => self.t(if self.state.session.is_some() {
                "Session confirmed"
            } else {
                "No session booked"
            }),
            10 => self.t("The agent debriefs Margaret"),
            11 => self.state.workflow.name.clone(),
            _ => self.t("Margaret's engine"),
        }
    }

    pub(crate) fn screen(&mut self, cx: &mut Context<Self>) -> AnyElement {
        match self.state.step {
            0 => self.welcome(cx),
            1 => self.conversation(0, cx),
            2 => self.profile(cx),
            3 => self.price(cx),
            4 => self.conversation(1, cx),
            5 => self.brief(cx),
            6 => self.match_screen(),
            7 => self.proposal(cx),
            8 => self.accept(cx),
            9 => self.session(cx),
            10 => self.conversation(2, cx),
            11 => self.workflow(cx),
            _ => self.engine(cx),
        }
    }

    fn welcome(&self, cx: &Context<Self>) -> AnyElement {
        let live = !self.checking
            && self.live_on
            && self
                .status
                .as_ref()
                .is_some_and(|status| status.state == StatusState::Ready);
        let disclosure = self.t(if live {
            "All data is scripted except the three conversations, which come from a live agent. No payments or calendars are connected."
        } else {
            "All data is scripted. No agents, payments or calendars are connected."
        });
        column(vec![
            copy("welcome-introduction", self.t("Agely is an expert network where older people sell decades of experience in 15-minute sessions. This walkthrough follows two fictional people from first conversation to first earnings.")),
            card("welcome-margaret", vec![
                heading("margaret-name", self.message("{0}, {1}", &["Margaret Okafor".into(), self.i18n.number(68)]), 3),
                copy("margaret-background", self.t("Retired procurement director from Columbus, Ohio. 34 years choosing suppliers and negotiating contracts.")),
                muted("margaret-role", self.t("The expert")),
            ]),
            card("welcome-dev", vec![
                heading("dev-name", "Dev Patel", 3),
                copy("dev-background", self.t("Hardware founder with a team of six, about to sign his first contract-manufacturing agreement.")),
                muted("dev-role", self.t("The buyer")),
            ]),
            notice("welcome-fictional-data", self.message("{0} People make every decision; agents only suggest.", &[disclosure]), false),
            self.live_agent_panel(cx),
            self.decision("start-walkthrough", "Start the walkthrough", Action::Next, cx),
        ])
    }

    fn live_agent_panel(&self, cx: &Context<Self>) -> AnyElement {
        let ready = !self.checking
            && self
                .status
                .as_ref()
                .is_some_and(|status| status.state == StatusState::Ready);
        let headline = if self.checking || self.status.is_none() {
            "Live agent: checking…"
        } else {
            match self.status.as_ref().map(|status| status.state) {
                Some(StatusState::Ready) if !self.live_on => "Live agent: ready, turned off",
                Some(StatusState::Ready) => "Live agent: ready",
                Some(StatusState::NotInstalled) => "Live agent: off — Codex is not installed",
                Some(StatusState::NotSignedIn) => "Live agent: off — Codex is not signed in",
                Some(StatusState::ModelUnavailable) => "Live agent: off — no allowed model",
                _ => "Live agent: off — the check failed",
            }
        };
        let mut children = vec![heading("agent-readiness", self.t(headline), 3)];
        if self.checking || self.status.is_none() {
            children.push(copy(
                "agent-checking",
                self.t("Checking whether the live agent can run on this computer…"),
            ));
        } else if let Some(status) = &self.status {
            let detail = if self.i18n.locale() == "en" {
                status.detail.clone()
            } else {
                match status.state {
                    StatusState::Ready => self.message("Live agent ready, using {0} through your Codex sign-in.", &[status.model.clone().unwrap_or_else(|| "Codex".into())]),
                    StatusState::NotInstalled => self.t("Codex is not installed on this computer, so the scripted conversations are used."),
                    StatusState::NotSignedIn => self.t("Codex is installed but not signed in."),
                    StatusState::ModelUnavailable => self.t("Your ChatGPT plan does not allow an available Codex model."),
                    StatusState::Error => self.t("The live agent check failed. The walkthrough uses scripted conversations."),
                }
            };
            children.push(copy("agent-status-detail", detail));
            if ready && self.live_on {
                let model = status
                    .model
                    .as_ref()
                    .map(|model| self.message(" with {0}", from_ref(model)))
                    .unwrap_or_default();
                children.push(muted("agent-privacy", self.message("The three conversations are generated live{0}. Their text is sent to OpenAI through the ChatGPT account signed in to Codex on this computer. No API key is used.", &[model])));
            }
            if !ready {
                children.push(muted(
                    "agent-scripted-fallback",
                    self.t("The walkthrough uses the scripted conversations."),
                ));
            }
            if let Some(fix) = &status.fix {
                let fix = if self.i18n.locale() == "en" {
                    fix.clone()
                } else {
                    match status.state {
                        StatusState::NotInstalled => self.message("Install it with: {0} — then select Check again.", &["npm install -g @openai/codex".into()]),
                        StatusState::NotSignedIn => self.message("In a terminal run: {0} — sign in with ChatGPT, then select Check again.", &["codex login".into()]),
                        StatusState::ModelUnavailable => self.message("Set {0} to a model your plan allows, restart the app, then Check again.", &["AGELY_CODEX_MODEL".into()]),
                        _ => self.t("Check your internet connection, then select Check again."),
                    }
                };
                children.push(copy("agent-recovery", self.message("To fix: {0}", &[fix])));
            }
            if self.i18n.locale() != "en"
                && matches!(
                    status.state,
                    StatusState::Error | StatusState::ModelUnavailable
                )
            {
                children.push(heading(
                    "agent-original-diagnostic-title",
                    self.t("Technical details (original language)"),
                    3,
                ));
                children.push(muted("agent-original-diagnostic", status.detail.clone()));
            }
        }
        let mut buttons = Vec::new();
        if ready {
            buttons.push(self.button(
                "toggle-live-agent",
                self.t(if self.live_on {
                    "Turn live agent off"
                } else {
                    "Turn live agent on"
                }),
                UiAction::ToggleLive,
                true,
                false,
                cx,
            ));
        }
        if !self.checking {
            buttons.push(self.button(
                "recheck-live-agent",
                self.t("Check again"),
                UiAction::Recheck,
                true,
                false,
                cx,
            ));
        }
        if !buttons.is_empty() {
            children.push(row(buttons));
        }
        card("live-agent-panel", children)
    }

    fn conversation(&mut self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        let lead = match index {
            0 => {
                "No forms. The agent interviews Margaret about her career and reads the documents she offers."
            }
            1 => {
                "Dev talks to his buyer's agent, which turns a business problem into a clear brief."
            }
            _ => {
                "The session with Dev has happened. The agent asks Margaret how she thinks, so her engine can learn it."
            }
        };
        let agent_name = self.t(match index {
            0 => "Onboarding agent",
            1 => "Buyer's agent",
            _ => "Agely agent",
        });
        let mut children = vec![copy("conversation-introduction", self.t(lead))];
        if self.is_live(index) {
            children.push(self.live_conversation(index, agent_name, cx));
        } else {
            let shown = self.state.shown[index];
            let turns = self.state.conversation_turns(index);
            let done = shown >= turns.len();
            for (position, turn) in turns.iter().take(shown).enumerate() {
                let speaker = match turn.speaker {
                    Speaker::Agent => agent_name.clone(),
                    Speaker::Margaret => "Margaret".into(),
                    Speaker::Dev => "Dev".into(),
                };
                children.push(card(
                    ("scripted-turn", position),
                    vec![
                        heading("speaker", speaker, 3),
                        copy("words", turn.text.clone()),
                    ],
                ));
            }
            children.push(muted(
                "scripted-voice-disclosure",
                self.t("This conversation stands in for a voice interview."),
            ));
            let button_label = self.t(if shown == 0 {
                "Start the conversation"
            } else if done {
                "Conversation finished"
            } else {
                "Next line"
            });
            children.push(row(vec![
                self.button(
                    "reveal-scripted-line",
                    button_label,
                    UiAction::Domain(Action::ShowNextTurn),
                    !done,
                    !done,
                    cx,
                ),
                muted(
                    "scripted-line-count",
                    self.message(
                        "{0} of {1} lines",
                        &[
                            self.i18n.number(shown.min(turns.len()) as i64),
                            self.i18n.number(turns.len() as i64),
                        ],
                    ),
                ),
            ]));
        }
        column(children)
    }

    fn live_conversation(
        &mut self,
        index: usize,
        agent_name: String,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let conversation = self.live[index].clone();
        let person = if index == 1 { "Dev" } else { "Margaret" };
        let pending = conversation.pending.is_some();
        let can_reply = !pending
            && !conversation.ended
            && conversation.error.is_none()
            && !conversation.lines.is_empty();
        let mut children = vec![muted(
            "live-conversation-label",
            self.t("Live agent (Codex)"),
        )];
        for (position, turn) in conversation.lines.iter().enumerate() {
            let speaker = if turn.speaker == AgentSpeaker::Agent {
                agent_name.clone()
            } else {
                person.into()
            };
            children.push(card(
                ("live-turn", position),
                vec![
                    heading("speaker", speaker, 3),
                    copy("words", turn.text.clone()),
                ],
            ));
        }
        if conversation.lines.is_empty()
            && !pending
            && conversation.error.is_none()
            && !conversation.ended
        {
            children.push(self.button(
                "start-live-conversation",
                self.t("Start the conversation"),
                UiAction::StartLive(index),
                true,
                true,
                cx,
            ));
        }
        if pending {
            children.push(notice(
                "live-thinking",
                self.t("The agent is thinking…"),
                false,
            ));
        }
        if let Some(error) = &conversation.error {
            children.push(notice(
                "live-error",
                self.t("The agent could not reply. Try again, or use the scripted conversation."),
                true,
            ));
            children.push(copy("live-error-detail", error.clone()));
            children.push(row(vec![
                self.button(
                    "retry-live-conversation",
                    self.t("Try again"),
                    UiAction::StartLive(index),
                    !conversation.ended,
                    false,
                    cx,
                ),
                self.button(
                    "live-scripted-fallback",
                    self.t("Use the scripted conversation"),
                    UiAction::Scripted(index),
                    true,
                    false,
                    cx,
                ),
            ]));
        }
        if can_reply {
            if !conversation.options.is_empty() {
                children.push(copy(
                    "reply-options-label",
                    self.message("{0} can say", &[person.into()]),
                ));
                let options = conversation
                    .options
                    .iter()
                    .enumerate()
                    .map(|(position, option)| {
                        self.button(
                            SharedString::from(format!("live-option-{position}")),
                            self.message("“{0}”", from_ref(option)),
                            UiAction::ReplyLive(index, option.clone()),
                            true,
                            false,
                            cx,
                        )
                    })
                    .collect();
                children.push(column(options));
            }
            let field_id = format!("live-reply-{index}");
            let has_reply = self
                .fields
                .get(&field_id)
                .is_some_and(|field| !field.read(cx).value().trim().is_empty());
            children.push(self.field(
                &field_id,
                self.message("Or type {0}'s reply", &[person.into()]),
                String::new(),
                cx,
            ));
            children.push(self.button(
                "send-live-reply",
                self.t("Send"),
                UiAction::SendLive(index),
                has_reply,
                false,
                cx,
            ));
        }
        if conversation.can_finish() {
            children.push(self.button(
                "finish-live-conversation",
                self.t("Finish conversation"),
                UiAction::FinishLive(index),
                true,
                false,
                cx,
            ));
        }
        if conversation.ended {
            children.push(notice(
                "live-conversation-finished",
                self.t("Conversation finished."),
                false,
            ));
        }
        column(children)
    }

    fn scripted_draft(&self, index: usize) -> Option<AnyElement> {
        (!self.live[index].scripted && !self.live[index].lines.is_empty()).then(|| {
            notice("scripted-draft-disclosure", self.t("This is the scripted draft. Drafting it from the live conversation comes later."), false)
        })
    }

    fn profile(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let expert = self.state.margaret.clone();
        let mut children = vec![copy(
            "profile-publication-state",
            self.t(if expert.published {
                "Published"
            } else {
                "Draft — not published"
            }),
        )];
        children.extend(self.scripted_draft(0));
        let name = expert
            .age
            .map(|age| self.message("{0}, {1}", &[expert.name.clone(), self.i18n.number(age)]))
            .unwrap_or_else(|| expert.name.clone());
        children.push(card(
            "profile-summary",
            vec![
                heading("profile-name", name, 3),
                copy("profile-headline", expert.headline),
                muted(
                    "profile-credentials",
                    self.message(
                        "{0} years of experience · {1} · Credentials are self-declared",
                        &[self.i18n.number(expert.years_experience), expert.location],
                    ),
                ),
            ],
        ));
        children.push(heading("profile-skills-title", self.t("Skills"), 3));
        let mut typed_edits = vec![heading(
            "typed-profile-edits-title",
            self.t("Or type the change yourself"),
            3,
        )];
        for competency in expert.competencies {
            let source = match competency.source {
                CompetencySource::Interview { turn } => {
                    self.message("From the interview (line {0})", &[self.i18n.number(turn)])
                }
                CompetencySource::Document { name } => self.message("From: {0}", &[name]),
            };
            let input_id = format!("skill-{}", competency.id);
            let edit = self.field(
                &input_id,
                self.message("Rename “{0}”", from_ref(&competency.label)),
                competency.label.clone(),
                cx,
            );
            children.push(card(
                SharedString::from(format!("competency-{}", competency.id)),
                vec![
                    heading("skill-name", competency.label, 3),
                    copy("skill-description", competency.description),
                    muted("skill-provenance", source),
                ],
            ));
            typed_edits.push(card(
                SharedString::from(format!("edit-competency-{}", competency.id)),
                vec![
                    edit,
                    row(vec![
                        self.button(
                            SharedString::from(format!("save-{}", competency.id)),
                            self.t("Save name"),
                            UiAction::RenameSkill(competency.id.clone()),
                            true,
                            false,
                            cx,
                        ),
                        self.button(
                            SharedString::from(format!("remove-{}", competency.id)),
                            self.t("Remove"),
                            UiAction::Domain(Action::RemoveCompetency { id: competency.id }),
                            true,
                            false,
                            cx,
                        ),
                    ]),
                ],
            ));
        }
        children.push(self.story_list(
            "profile-examples",
            "Problems Margaret can help with",
            &self.state.margaret.example_problems,
        ));
        children.push(copy(
            "profile-prompt",
            self.t("Anything you'd like to change before I publish?"),
        ));
        children.extend(self.exchanges(0));
        let replies = self
            .state
            .seed
            .profile_replies
            .iter()
            .map(|reply| {
                self.button(
                    SharedString::from(reply.id.clone()),
                    self.message("“{0}”", from_ref(&reply.text)),
                    UiAction::Domain(Action::ReplyToAgent {
                        reply_id: reply.id.clone(),
                    }),
                    true,
                    false,
                    cx,
                )
            })
            .collect();
        children.push(column(replies));
        if !self.state.margaret.published {
            children.push(self.decision(
                "publish-profile",
                "Publish my profile",
                Action::PublishProfile,
                cx,
            ));
        }
        children.push(self.button(
            "redo-interview",
            self.t("Start the interview again"),
            UiAction::Domain(Action::RedoInterview),
            true,
            false,
            cx,
        ));
        children.extend(typed_edits);
        column(children)
    }

    fn exchanges(&self, index: usize) -> Option<AnyElement> {
        if self.state.exchanges[index].is_empty() {
            return None;
        }
        let turns = self.state.exchanges[index]
            .iter()
            .enumerate()
            .map(|(position, turn)| {
                let speaker = if turn.speaker == Speaker::Agent {
                    self.t("Onboarding agent")
                } else {
                    "Margaret".into()
                };
                card(
                    ("exchange", position),
                    vec![
                        heading("speaker", speaker, 3),
                        copy("words", turn.text.clone()),
                    ],
                )
            })
            .collect();
        Some(card("agent-exchanges", vec![column(turns)]))
    }

    fn price(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let suggestion = self.state.seed.price_suggestion.clone();
        let mut proposed = vec![
            muted("price-suggestion-label", self.t("The agent suggests")),
            heading(
                "price-suggestion-range",
                self.message("{0} per 15 minutes", &[self.range(suggestion.range)]),
                3,
            ),
            heading("price-reasons-label", self.t("Why"), 3),
        ];
        proposed.extend(
            suggestion
                .reasons
                .into_iter()
                .enumerate()
                .map(|(index, reason)| copy(("price-reason", index), reason)),
        );
        let mut children = vec![
            copy(
                "price-introduction",
                self.t("The agent suggests a price per 15 minutes. Margaret decides."),
            ),
            card("price-suggestion", proposed),
            copy(
                "price-question",
                self.t("Does that price feel right to you?"),
            ),
        ];
        children.extend(self.exchanges(1));
        let replies = self
            .state
            .seed
            .price_replies
            .iter()
            .map(|reply| {
                self.button(
                    SharedString::from(reply.id.clone()),
                    self.message("“{0}”", from_ref(&reply.text)),
                    UiAction::Domain(Action::ReplyToAgent {
                        reply_id: reply.id.clone(),
                    }),
                    true,
                    false,
                    cx,
                )
            })
            .collect();
        children.push(column(replies));
        let minimum = self.field(
            "price-min",
            self.t("Lowest price per 15 minutes ($)"),
            self.state.price_draft.min.to_string(),
            cx,
        );
        let maximum = self.field(
            "price-max",
            self.t("Highest price per 15 minutes ($)"),
            self.state.price_draft.max.to_string(),
            cx,
        );
        let custom_price = card(
            "custom-price",
            vec![
                heading("custom-price-title", self.t("Or type your own range"), 3),
                minimum,
                maximum,
                self.button(
                    "save-custom-price",
                    self.t("Use this range"),
                    UiAction::SavePrice,
                    true,
                    false,
                    cx,
                ),
            ],
        );
        children.push(card(
            "current-price",
            vec![
                copy("current-price-label", self.t("Margaret's price:")),
                copy(
                    "current-price-range",
                    self.message("{0} per 15 minutes", &[self.range(self.state.price_draft)]),
                ),
                copy(
                    "current-price-confirmation",
                    self.t(if self.state.price_confirmed {
                        "Confirmed"
                    } else {
                        "Not confirmed yet"
                    }),
                ),
            ],
        ));
        if !self.state.price_confirmed {
            children.push(self.decision(
                "confirm-price",
                "Confirm my price",
                Action::ConfirmPrice,
                cx,
            ));
        }
        children.push(custom_price);
        column(children)
    }

    fn brief(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let brief = self.state.brief.clone();
        let mut children = vec![copy(
            "brief-confirmation-state",
            self.t(if brief.confirmed {
                "Confirmed"
            } else {
                "Draft — check and confirm"
            }),
        )];
        children.extend(self.scripted_draft(1));
        children.push(self.field(
            "brief-problem",
            self.t("The problem, in one sentence"),
            brief.problem,
            cx,
        ));
        children.push(card(
            "brief-details",
            vec![
                heading("brief-competency-title", self.t("Competency needed"), 3),
                copy("brief-competency", brief.competency_label),
                heading("brief-length-title", self.t("Session length"), 3),
                copy(
                    "brief-length",
                    self.message(
                        "{0} minutes ({1} × 15 min)",
                        &[
                            self.i18n.number(brief.units * 15),
                            self.i18n.number(brief.units),
                        ],
                    ),
                ),
            ],
        ));
        children.push(self.field(
            "budget-min",
            self.t("Budget from, per 15 minutes ($)"),
            brief.budget.min.to_string(),
            cx,
        ));
        children.push(self.field(
            "budget-max",
            self.t("Budget up to, per 15 minutes ($)"),
            brief.budget.max.to_string(),
            cx,
        ));
        children.push(muted(
            "brief-budget",
            self.message("Budget: {0} per 15 minutes", &[self.range(brief.budget)]),
        ));
        children.push(notice("commercial-terms-disclosure", self.t("Agely doesn't give legal advice. This brief covers the commercial terms; for a legal review, use a lawyer."), false));
        if !brief.confirmed {
            children.push(self.button(
                "confirm-brief",
                self.t("Confirm my brief"),
                UiAction::ConfirmBrief,
                true,
                true,
                cx,
            ));
        }
        column(children)
    }

    fn match_screen(&self) -> AnyElement {
        let mut children = Vec::new();
        let others = match self.state.current_match() {
            MatchResult::Match {
                expert,
                overlap,
                explanation,
                others,
                ..
            } => {
                let mut details = vec![
                    heading("matched-expert-name", expert.name, 3),
                    copy("matched-expert-headline", expert.headline),
                    heading(
                        "matched-price-range",
                        self.message("Price overlap: {0} per 15 minutes", &[self.range(overlap)]),
                        3,
                    ),
                    heading("match-explanation-title", self.t("Why this match"), 3),
                ];
                details.extend(explanation.iter().enumerate().map(|(index, reason)| {
                    copy(("match-reason", index), self.notice_text(reason))
                }));
                children.push(card("matched-expert", details));
                others
            }
            MatchResult::None {
                explanation,
                others,
                ..
            } => {
                children.push(notice(
                    "no-match-explanation",
                    self.notice_text(&explanation),
                    false,
                ));
                others
            }
        };
        if !others.is_empty() {
            let mut reasons = vec![heading(
                "other-experts-title",
                self.t("Why not the others"),
                3,
            )];
            for (index, other) in others.iter().enumerate() {
                let explanation = match other.reason {
                    NotMatchedReason::Competency => self.message(
                        "Doesn't list “{0}”.",
                        from_ref(&self.state.brief.competency_label),
                    ),
                    NotMatchedReason::Price => self.message(
                        "Charges {0} per 15 minutes, outside your budget.",
                        &[other
                            .expert
                            .price_range
                            .map(|range| self.range(range))
                            .unwrap_or_else(|| self.t("an unknown price"))],
                    ),
                    NotMatchedReason::Unpublished => {
                        self.t("Profile or price isn't published yet.")
                    }
                    NotMatchedReason::Ranked => {
                        self.t("Also a fit, but with a narrower price overlap or less experience.")
                    }
                };
                reasons.push(copy(
                    ("unmatched-expert", index),
                    self.message("{0}: {1}", &[other.expert.name.clone(), explanation]),
                ));
            }
            children.push(card("other-experts", reasons));
        }
        column(children)
    }

    fn proposal(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let MatchResult::Match { overlap, .. } = self.state.current_match() else {
            return copy(
                "proposal-needs-match",
                self.t("Go back to the match step to see why."),
            );
        };
        let current_price = self
            .fields
            .get("proposal-price")
            .map_or(Some(self.state.proposal.price_per_unit), |field| {
                parse_dollars(field.read(cx).value())
            });
        let check = current_price.map(|price_per_unit| {
            validate_proposal(
                &Proposal {
                    units: self.state.proposal.units,
                    price_per_unit,
                    sent: self.state.proposal.sent,
                },
                overlap,
            )
        });
        let choices = (1..=4)
            .map(|units| {
                self.button(
                    SharedString::from(format!("proposal-length-{units}")),
                    self.message("{0} min", &[self.i18n.number(units * 15)]),
                    UiAction::Domain(Action::SetProposalUnits { units }),
                    true,
                    self.state.proposal.units == units,
                    cx,
                )
            })
            .collect();
        let price_field = self.field(
            "proposal-price",
            self.message(
                "Price per 15 minutes ($), between {0}",
                &[self.range(overlap)],
            ),
            self.state.proposal.price_per_unit.to_string(),
            cx,
        );
        let mut children = vec![
            copy(
                "proposal-introduction",
                self.t(
                    "The two of them agree the details. Nothing is booked until Margaret accepts.",
                ),
            ),
            heading("proposal-length-label", self.t("Length"), 3),
            row(choices),
            price_field,
        ];
        let total = match check {
            Some(Ok(total)) => self.i18n.money(total),
            Some(Err(error)) => {
                children.push(notice(
                    "proposal-price-error",
                    self.notice_text(&error),
                    true,
                ));
                self.t("—")
            }
            None => {
                children.push(notice(
                    "proposal-price-error",
                    self.t("Please enter a whole-dollar price of at least $1."),
                    true,
                ));
                self.t("—")
            }
        };
        children.push(heading(
            "proposal-total",
            self.message(
                "Total: {0} for {1} minutes",
                &[total, self.i18n.number(self.state.proposal.units * 15)],
            ),
            3,
        ));
        if self.state.proposal.sent {
            children.push(notice(
                "proposal-waiting",
                self.t("Sent to Margaret. Waiting for her answer."),
                false,
            ));
        } else {
            children.push(self.button(
                "send-proposal",
                self.t("Send proposal to Margaret"),
                UiAction::SendProposal,
                true,
                true,
                cx,
            ));
        }
        column(children)
    }

    fn accept(&self, cx: &Context<Self>) -> AnyElement {
        let proposal = &self.state.proposal;
        let total = proposal
            .units
            .checked_mul(proposal.price_per_unit)
            .map(|amount| self.i18n.money(amount))
            .unwrap_or_else(|| self.t("—"));
        let mut children = vec![card(
            "received-proposal",
            vec![
                copy(
                    "received-proposal-amounts",
                    self.message(
                        "Dev Patel asks for {0} minutes at {1} per 15 minutes: {2} in total.",
                        &[
                            self.i18n.number(proposal.units * 15),
                            self.i18n.money(proposal.price_per_unit),
                            total,
                        ],
                    ),
                ),
                muted(
                    "received-proposal-problem",
                    self.message("“{0}”", from_ref(&self.state.brief.problem)),
                ),
            ],
        )];
        if self.state.session.is_none() {
            children.push(notice(
                "acceptance-disclosure",
                self.t("Nothing is booked or charged until Margaret accepts."),
                false,
            ));
            children.push(row(vec![
                self.decision("accept-proposal", "Accept", Action::AcceptProposal, cx),
                self.button(
                    "decline-proposal",
                    self.t("Decline"),
                    UiAction::Domain(Action::DeclineProposal),
                    true,
                    false,
                    cx,
                ),
            ]));
        } else {
            children.push(notice(
                "proposal-accepted",
                self.t("Margaret accepted."),
                false,
            ));
        }
        column(children)
    }

    fn session(&self, cx: &Context<Self>) -> AnyElement {
        let Some(session) = &self.state.session else {
            return column(Vec::new());
        };
        let held = session.status == SessionStatus::Held;
        let mut children = vec![
            card(
                "session-time",
                vec![
                    heading("session-time-title", self.t("When"), 3),
                    copy("session-date", self.session_time()),
                    copy(
                        "session-duration",
                        self.message("{0} minutes", &[self.i18n.number(session.units * 15)]),
                    ),
                ],
            ),
            card(
                "session-price",
                vec![
                    heading("session-price-title", self.t("Price"), 3),
                    copy(
                        "session-price-calculation",
                        self.message(
                            "{0} ({1} × {2})",
                            &[
                                self.i18n.money(session.total),
                                self.i18n.number(session.units),
                                self.i18n.money(session.price_per_unit),
                            ],
                        ),
                    ),
                ],
            ),
            card(
                "session-meeting",
                vec![
                    heading("session-provider", session.call.provider.clone(), 3),
                    copy("session-demo-address", session.call.url.clone()),
                    muted(
                        "session-meeting-disclosure",
                        self.t("Demo link — not a real meeting"),
                    ),
                ],
            ),
            card(
                "session-payment",
                vec![
                    heading(
                        "session-payment-disclosure",
                        self.t("Payment (demo — no money moves)"),
                        3,
                    ),
                    notice(
                        "session-payment-state",
                        self.t(if held {
                            "Released to Margaret"
                        } else {
                            "Payment held until the session takes place"
                        }),
                        false,
                    ),
                ],
            ),
        ];
        if !held {
            children.push(self.decision(
                "hold-session",
                "Mark the session as held",
                Action::MarkSessionHeld,
                cx,
            ));
        }
        column(children)
    }

    fn workflow(&self, cx: &Context<Self>) -> AnyElement {
        let workflow = &self.state.workflow;
        let approved = workflow.status == WorkflowStatus::Approved;
        let mut children = vec![
            copy("workflow-approval-state", self.t(if approved { "Approved" } else { "Proposed" })),
            copy("workflow-introduction", self.t("The agent wrote this from Margaret's debrief. It becomes part of her engine only if she approves it.")),
        ];
        children.extend(self.scripted_draft(2));
        children.push(self.story_list(
            "workflow-questions",
            "Questions Margaret asks",
            &workflow.questions,
        ));
        children.push(self.story_list("workflow-checks", "Checks she runs", &workflow.checks));
        children.push(self.story_list(
            "workflow-watch-outs",
            "Mistakes she watches for",
            &workflow.watch_outs,
        ));
        if !approved {
            children.push(self.decision(
                "approve-workflow",
                "Approve for my engine",
                Action::ApproveWorkflow,
                cx,
            ));
        }
        column(children)
    }

    fn engine(&self, cx: &Context<Self>) -> AnyElement {
        let engine = self.state.derive_engine();
        let workflows = i64::try_from(engine.workflows.len()).unwrap_or(i64::MAX);
        let workflow_label = if self.i18n.locale() == "en" {
            if workflows == 1 {
                "workflow".into()
            } else {
                "workflows".into()
            }
        } else {
            self.t("Workflows")
        };
        let session_label = if self.i18n.locale() == "en" {
            if engine.sessions_completed == 1 {
                "session completed".into()
            } else {
                "sessions completed".into()
            }
        } else {
            self.t("Sessions completed")
        };
        let mut children = vec![
            copy("engine-introduction", self.t("Every session teaches the engine how Margaret thinks. Her judgement keeps working after her career.")),
            card("engine-workflow-count", vec![heading("value", self.i18n.number(workflows), 3), copy("label", workflow_label)]),
            card("engine-session-count", vec![heading("value", self.i18n.number(engine.sessions_completed), 3), copy("label", session_label)]),
            card("engine-earnings", vec![heading("value", self.i18n.money(engine.earnings), 3), copy("label", self.t("earned"))]),
            heading("engine-approved-title", self.t("Approved workflows"), 3),
        ];
        if engine.workflows.is_empty() {
            children.push(muted("engine-no-workflows", self.t("None yet.")));
        } else {
            children.extend(
                engine
                    .workflows
                    .iter()
                    .enumerate()
                    .map(|(index, workflow)| {
                        copy(("approved-workflow", index), workflow.name.clone())
                    }),
            );
        }
        let roadmap = [
            (
                "Today",
                "The engine confirms Margaret's level and prepares her for each session.",
            ),
            (
                "Next",
                "It carries out follow-up work the way she would, and she reviews it.",
            ),
            (
                "Then",
                "Buyers consult the engine directly, within limits Margaret sets, so she earns beyond her hours.",
            ),
            (
                "Later",
                "Engines from several experts work together on problems no one person covers.",
            ),
        ];
        let mut plans = vec![heading(
            "engine-roadmap-title",
            self.t("Where the engine goes next"),
            3,
        )];
        plans.extend(
            roadmap
                .into_iter()
                .enumerate()
                .map(|(index, (when, what))| {
                    column(vec![
                        heading(
                            ("roadmap-when", index),
                            self.message("{0}:", &[self.t(when)]),
                            3,
                        ),
                        copy(("roadmap-detail", index), self.t(what)),
                    ])
                }),
        );
        children.push(card("engine-roadmap", plans));
        children.push(self.button(
            "repeat-walkthrough",
            self.t("Run the walkthrough again"),
            UiAction::Domain(Action::Restart),
            true,
            false,
            cx,
        ));
        column(children)
    }

    fn story_list(&self, id: &'static str, title: &str, items: &[String]) -> AnyElement {
        let mut children = vec![heading("list-title", self.t(title), 3)];
        children.extend(
            items
                .iter()
                .enumerate()
                .map(|(index, item)| copy(("item", index), item.clone())),
        );
        card(id, children)
    }
}
