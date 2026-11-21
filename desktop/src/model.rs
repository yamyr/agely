//! Pure walkthrough state shared by the native screens. Decisions remain explicit.

use regex::Regex;
use serde::Deserialize;
use std::sync::LazyLock;
use unicode_normalization::UnicodeNormalization;

pub const STEP_TITLES: [&str; 13] = [
    "Meet Margaret and Dev",
    "Margaret talks to the onboarding agent",
    "Margaret reviews her draft profile",
    "Margaret sets her price",
    "Dev describes his problem",
    "Dev checks his brief",
    "Agely finds a match",
    "Dev proposes a session",
    "Margaret answers Dev's proposal",
    "The session is booked",
    "The agent debriefs Margaret",
    "Margaret reviews a new workflow",
    "Margaret's engine",
];

pub const CHAPTER_NAMES: [&str; 7] = [
    "Margaret onboards",
    "Price",
    "Dev's brief",
    "Match",
    "Book & pay",
    "Debrief",
    "Engine",
];
pub const CHAPTER_STEPS: [usize; 7] = [1, 3, 4, 6, 7, 10, 12];

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Speaker {
    Agent,
    Margaret,
    Dev,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ConversationTurn {
    pub speaker: Speaker,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum CompetencySource {
    Interview { turn: i64 },
    Document { name: String },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Competency {
    pub id: String,
    pub tag: String,
    pub label: String,
    pub description: String,
    pub source: CompetencySource,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub struct PriceRange {
    pub min: i64,
    pub max: i64,
}

impl PriceRange {
    pub fn overlap(self, other: Self) -> Option<Self> {
        let shared = Self {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        };
        (shared.min <= shared.max).then_some(shared)
    }

    pub fn contains(self, amount: i64) -> bool {
        (self.min..=self.max).contains(&amount)
    }

    pub fn validate(self) -> Result<Self, Notice> {
        if self.min < 1 || self.max < 1 {
            return Err(Notice::error("Both prices must be at least $1."));
        }
        if self.min > self.max {
            return Err(Notice::with_args(
                "The lowest price ({0}) is higher than the highest ({1}). Swap them, or change one.",
                vec![NoticeArg::Money(self.min), NoticeArg::Money(self.max)],
            ));
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct PriceSuggestion {
    pub range: PriceRange,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Expert {
    pub id: String,
    pub name: String,
    pub age: Option<i64>,
    pub headline: String,
    pub years_experience: i64,
    pub location: String,
    pub competencies: Vec<Competency>,
    pub example_problems: Vec<String>,
    pub price_range: Option<PriceRange>,
    pub published: bool,
    pub credentials_self_declared: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Brief {
    pub problem: String,
    pub competency_tag: String,
    pub competency_label: String,
    pub units: i64,
    pub budget: PriceRange,
    pub confirmed: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowStatus {
    Proposed,
    Approved,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Workflow {
    pub name: String,
    pub questions: Vec<String>,
    pub checks: Vec<String>,
    pub watch_outs: Vec<String>,
    pub status: WorkflowStatus,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ReplyEffect {
    RenameCompetency { id: String, label: String },
    RemoveCompetency { id: String },
    SetPrice { range: PriceRange },
    None,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct SuggestedReply {
    pub id: String,
    pub text: String,
    pub ack: String,
    pub effect: ReplyEffect,
}

/// Exported from the authored browser story, then localized before a run starts.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Seed {
    pub interview_turns: Vec<ConversationTurn>,
    pub margaret: Expert,
    pub price_suggestion: PriceSuggestion,
    pub profile_replies: Vec<SuggestedReply>,
    pub price_replies: Vec<SuggestedReply>,
    pub brief_turns: Vec<ConversationTurn>,
    pub brief: Brief,
    pub other_experts: Vec<Expert>,
    pub debrief_turns: Vec<ConversationTurn>,
    pub workflow: Workflow,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoticeArg {
    Text(String),
    Number(i64),
    Money(i64),
    Range(PriceRange),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoticeKind {
    Error,
    Info,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    pub kind: NoticeKind,
    pub source: String,
    pub args: Vec<NoticeArg>,
}

impl Notice {
    pub fn error(source: impl Into<String>) -> Self {
        Self::with_args(source, Vec::new())
    }

    pub fn with_args(source: impl Into<String>, args: Vec<NoticeArg>) -> Self {
        Self {
            kind: NoticeKind::Error,
            source: source.into(),
            args,
        }
    }

    pub fn info(source: impl Into<String>) -> Self {
        Self {
            kind: NoticeKind::Info,
            source: source.into(),
            args: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotMatchedReason {
    Competency,
    Price,
    Unpublished,
    Ranked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotMatched {
    pub expert: Expert,
    pub reason: NotMatchedReason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MatchResult {
    Match {
        expert: Box<Expert>,
        matched_competencies: Vec<Competency>,
        overlap: PriceRange,
        explanation: Vec<Notice>,
        others: Vec<NotMatched>,
    },
    None {
        reason: NotMatchedReason,
        explanation: Notice,
        others: Vec<NotMatched>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    pub units: i64,
    pub price_per_unit: i64,
    pub sent: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionStatus {
    Accepted,
    Held,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaymentStatus {
    Held,
    Released,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Payment {
    pub amount: i64,
    pub status: PaymentStatus,
    pub demo: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeetingLink {
    pub url: String,
    pub provider: String,
    pub is_demo: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    pub id: String,
    pub units: i64,
    pub price_per_unit: i64,
    pub total: i64,
    pub call: MeetingLink,
    pub status: SessionStatus,
    pub payment: Payment,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Engine {
    pub workflows: Vec<Workflow>,
    pub sessions_completed: i64,
    pub earnings: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Next,
    Back,
    Restart,
    JumpTo {
        chapter: usize,
    },
    ShowNextTurn,
    ReplyToAgent {
        reply_id: String,
    },
    EditCompetency {
        id: String,
        label: String,
    },
    RemoveCompetency {
        id: String,
    },
    RedoInterview,
    PublishProfile,
    SetPrice {
        range: PriceRange,
    },
    ConfirmPrice,
    EditBrief {
        problem: String,
        units: i64,
        budget: PriceRange,
    },
    ConfirmBrief,
    SetProposalUnits {
        units: i64,
    },
    SetProposalPrice {
        price_per_unit: i64,
    },
    SendProposal,
    AcceptProposal,
    DeclineProposal,
    MarkSessionHeld,
    ApproveWorkflow,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DemoState {
    pub seed: Seed,
    pub step: usize,
    pub margaret: Expert,
    pub price_draft: PriceRange,
    pub price_confirmed: bool,
    pub brief: Brief,
    pub proposal: Proposal,
    pub session: Option<Session>,
    pub workflow: Workflow,
    pub shown: [usize; 3],
    pub exchanges: [Vec<ConversationTurn>; 2],
    pub notice: Option<Notice>,
}

impl DemoState {
    pub fn new(seed: Seed) -> Self {
        Self {
            step: 0,
            margaret: seed.margaret.clone(),
            price_draft: seed.price_suggestion.range,
            price_confirmed: false,
            brief: seed.brief.clone(),
            proposal: Proposal {
                units: seed.brief.units,
                price_per_unit: 75,
                sent: false,
            },
            session: None,
            workflow: seed.workflow.clone(),
            shown: [0; 3],
            exchanges: [Vec::new(), Vec::new()],
            notice: None,
            seed,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.seed.clone());
    }

    pub fn conversation_index(&self) -> Option<usize> {
        match self.step {
            1 => Some(0),
            4 => Some(1),
            10 => Some(2),
            _ => None,
        }
    }

    pub fn conversation_turns(&self, index: usize) -> &[ConversationTurn] {
        match index {
            0 => &self.seed.interview_turns,
            1 => &self.seed.brief_turns,
            2 => &self.seed.debrief_turns,
            _ => &[],
        }
    }

    pub fn can_advance(&self) -> bool {
        match self.step {
            0 => true,
            1 | 4 | 10 => self
                .conversation_index()
                .is_some_and(|index| self.shown[index] >= self.conversation_turns(index).len()),
            2 => self.margaret.published,
            3 => self.price_confirmed,
            5 => self.brief.confirmed,
            6 => matches!(self.current_match(), MatchResult::Match { .. }),
            7 => self.proposal.sent,
            8 => self.session.is_some(),
            9 => self
                .session
                .as_ref()
                .is_some_and(|session| session.status == SessionStatus::Held),
            11 => self.workflow.status == WorkflowStatus::Approved,
            _ => false,
        }
    }

    pub fn blocked_reason(&self) -> Option<&'static str> {
        if self.can_advance() {
            return None;
        }
        match self.step {
            1 | 4 | 10 => Some("Show every line of the conversation first."),
            2 => Some("Margaret needs to publish her profile first."),
            3 => Some("Margaret needs to confirm her price first."),
            5 => Some("Dev needs to confirm his brief first."),
            6 => Some("There is no match to continue with."),
            7 => Some("Dev needs to send his proposal first."),
            8 => Some("Margaret needs to answer the proposal first."),
            9 => Some("Mark the session as held first."),
            11 => Some("Margaret needs to approve the workflow first."),
            12 => Some("This is the end of the walkthrough."),
            _ => None,
        }
    }

    pub fn current_match(&self) -> MatchResult {
        let experts = std::iter::once(&self.margaret).chain(&self.seed.other_experts);
        match_brief(&self.brief, experts)
    }

    pub fn derive_engine(&self) -> Engine {
        let held = self.session.as_ref().filter(|session| {
            session.status == SessionStatus::Held
                && session.payment.status == PaymentStatus::Released
        });
        Engine {
            workflows: if self.workflow.status == WorkflowStatus::Approved {
                vec![self.workflow.clone()]
            } else {
                Vec::new()
            },
            sessions_completed: i64::from(held.is_some()),
            earnings: held.map_or(0, |session| session.total),
        }
    }

    /// Presenter chapter jumps replay the same scripted decisions as the browser walkthrough.
    pub fn jump(&mut self, chapter: usize) {
        let Some(&target) = CHAPTER_STEPS.get(chapter) else {
            return;
        };
        self.reset();
        for step in 0..target {
            self.step = step;
            match step {
                1 => self.shown[0] = self.seed.interview_turns.len(),
                2 => self.dispatch(Action::PublishProfile),
                3 => self.dispatch(Action::ConfirmPrice),
                4 => self.shown[1] = self.seed.brief_turns.len(),
                5 => self.dispatch(Action::ConfirmBrief),
                7 => self.dispatch(Action::SendProposal),
                8 => self.dispatch(Action::AcceptProposal),
                9 => self.dispatch(Action::MarkSessionHeld),
                10 => self.shown[2] = self.seed.debrief_turns.len(),
                11 => self.dispatch(Action::ApproveWorkflow),
                _ => {}
            }
        }
        self.step = target;
        self.notice = None;
    }

    pub fn dispatch(&mut self, action: Action) {
        match action {
            Action::Next => {
                if self.can_advance() && self.step < STEP_TITLES.len() - 1 {
                    self.step += 1;
                    self.notice = None;
                }
            }
            Action::Back => {
                if self.step > 0 {
                    self.step -= 1;
                    self.notice = None;
                }
            }
            Action::Restart => self.reset(),
            Action::JumpTo { chapter } => self.jump(chapter),
            Action::ShowNextTurn => {
                if let Some(index) = self.conversation_index() {
                    self.shown[index] =
                        (self.shown[index] + 1).min(self.conversation_turns(index).len());
                }
            }
            Action::ReplyToAgent { reply_id } => self.reply_to_agent(&reply_id),
            Action::EditCompetency { id, label } => {
                let label = label.trim();
                if label.is_empty() {
                    self.notice = Some(Notice::error(
                        "A skill needs a name. Type a few words, or remove the skill instead.",
                    ));
                    return;
                }
                let Some(competency) = self
                    .margaret
                    .competencies
                    .iter_mut()
                    .find(|competency| competency.id == id)
                else {
                    self.missing_competency();
                    return;
                };
                competency.label = label.into();
                self.profile_edited();
            }
            Action::RemoveCompetency { id } => {
                let Some(index) = self
                    .margaret
                    .competencies
                    .iter()
                    .position(|competency| competency.id == id)
                else {
                    self.missing_competency();
                    return;
                };
                self.margaret.competencies.remove(index);
                self.profile_edited();
            }
            Action::RedoInterview => {
                self.step = 1;
                self.shown[0] = 0;
                self.margaret = self.seed.margaret.clone();
                self.exchanges = [Vec::new(), Vec::new()];
                self.price_draft = self.seed.price_suggestion.range;
                self.price_confirmed = false;
                self.notice = None;
            }
            Action::PublishProfile => {
                if self.margaret.competencies.len() < 3 {
                    self.notice = Some(Notice::with_args(
                        "Keep at least {0} skills on your profile before publishing.",
                        vec![NoticeArg::Number(3)],
                    ));
                } else {
                    self.margaret.published = true;
                    self.notice = None;
                }
            }
            Action::SetPrice { range } => match range.validate() {
                Ok(range) => {
                    self.price_draft = range;
                    self.price_confirmed = false;
                    self.margaret.price_range = None;
                    self.notice = None;
                }
                Err(notice) => self.notice = Some(notice),
            },
            Action::ConfirmPrice => match self.price_draft.validate() {
                Ok(range) => {
                    self.price_confirmed = true;
                    self.margaret.price_range = Some(range);
                    self.notice = None;
                }
                Err(notice) => self.notice = Some(notice),
            },
            Action::EditBrief {
                problem,
                units,
                budget,
            } => {
                self.brief.problem = problem;
                self.brief.units = units;
                self.brief.budget = budget;
                self.brief.confirmed = false;
                self.notice = None;
            }
            Action::ConfirmBrief => self.confirm_brief(),
            Action::SetProposalUnits { units } => {
                if (1..=4).contains(&units) {
                    self.proposal.units = units;
                    self.proposal.sent = false;
                    self.notice = None;
                }
            }
            Action::SetProposalPrice { price_per_unit } => {
                self.proposal.price_per_unit = price_per_unit;
                self.proposal.sent = false;
                self.notice = None;
            }
            Action::SendProposal => {
                let MatchResult::Match { overlap, .. } = self.current_match() else {
                    self.notice = Some(Notice::error("There is no match to send a proposal to."));
                    return;
                };
                match validate_proposal(&self.proposal, overlap) {
                    Ok(_) => {
                        self.proposal.sent = true;
                        self.notice = None;
                    }
                    Err(notice) => self.notice = Some(notice),
                }
            }
            Action::AcceptProposal => self.accept_proposal(),
            Action::DeclineProposal => {
                if self.proposal.sent && self.session.is_none() {
                    self.step = 7;
                    self.proposal.sent = false;
                    self.notice = Some(Notice::info(
                        "Margaret declined this time. Nothing was booked or charged. You can propose again.",
                    ));
                }
            }
            Action::MarkSessionHeld => {
                if let Some(session) = self
                    .session
                    .as_mut()
                    .filter(|session| session.status == SessionStatus::Accepted)
                {
                    session.status = SessionStatus::Held;
                    session.payment.status = PaymentStatus::Released;
                    self.notice = None;
                }
            }
            Action::ApproveWorkflow => {
                self.workflow.status = WorkflowStatus::Approved;
                self.notice = None;
            }
        }
    }

    fn missing_competency(&mut self) {
        self.notice = Some(Notice::error("That skill isn't on the profile any more."));
    }

    fn profile_edited(&mut self) {
        self.notice = self.margaret.published.then(|| {
            Notice::info(
                "Your profile is back to draft. Publish it again when you're happy with it.",
            )
        });
        self.margaret.published = false;
    }

    fn reply_to_agent(&mut self, reply_id: &str) {
        let reply = self
            .seed
            .profile_replies
            .iter()
            .chain(&self.seed.price_replies)
            .find(|reply| reply.id == reply_id)
            .cloned();
        let Some(reply) = reply else {
            return;
        };
        let bucket = usize::from(
            !self
                .seed
                .profile_replies
                .iter()
                .any(|item| item.id == reply_id),
        );
        match reply.effect {
            ReplyEffect::RenameCompetency { id, label } => {
                self.dispatch(Action::EditCompetency { id, label })
            }
            ReplyEffect::RemoveCompetency { id } => self.dispatch(Action::RemoveCompetency { id }),
            ReplyEffect::SetPrice { range } => self.dispatch(Action::SetPrice { range }),
            ReplyEffect::None => self.notice = None,
        }
        if self
            .notice
            .as_ref()
            .is_some_and(|notice| notice.kind == NoticeKind::Error)
        {
            return;
        }
        self.exchanges[bucket].extend([
            ConversationTurn {
                speaker: Speaker::Margaret,
                text: reply.text,
            },
            ConversationTurn {
                speaker: Speaker::Agent,
                text: reply.ack,
            },
        ]);
    }

    fn confirm_brief(&mut self) {
        if self.brief.problem.trim().is_empty() {
            self.notice = Some(Notice::error("Describe the problem in a sentence first."));
            return;
        }
        if !(1..=4).contains(&self.brief.units) {
            self.notice = Some(Notice::error("Choose a length between 15 and 60 minutes."));
            return;
        }
        if let Err(notice) = self.brief.budget.validate() {
            self.notice = Some(notice);
            return;
        }
        if let Some(category) = detect_restricted_advice(&self.brief.problem) {
            self.notice = Some(Notice::error(category.message()));
            return;
        }
        self.brief.confirmed = true;
        self.proposal.units = self.brief.units;
        self.proposal.sent = false;
        self.notice = None;
    }

    fn accept_proposal(&mut self) {
        if !self.proposal.sent || self.session.is_some() {
            return;
        }
        let MatchResult::Match { overlap, .. } = self.current_match() else {
            self.notice = Some(Notice::error("There is no match to send a proposal to."));
            return;
        };
        let total = match validate_proposal(&self.proposal, overlap) {
            Ok(total) => total,
            Err(notice) => {
                self.notice = Some(notice);
                return;
            }
        };
        self.session = Some(Session {
            id: "s1".into(),
            units: self.proposal.units,
            price_per_unit: self.proposal.price_per_unit,
            total,
            call: MeetingLink {
                url: "https://meet.example.com/agely-demo-s1".into(),
                provider: "Google Meet".into(),
                is_demo: true,
            },
            status: SessionStatus::Accepted,
            payment: Payment {
                amount: total,
                status: PaymentStatus::Held,
                demo: true,
            },
        });
        self.notice = None;
    }
}

pub fn validate_proposal(proposal: &Proposal, shared: PriceRange) -> Result<i64, Notice> {
    if proposal.price_per_unit < 1 || !(1..=4).contains(&proposal.units) {
        return Err(Notice::error(
            "Please enter a whole-dollar price of at least $1.",
        ));
    }
    if !shared.contains(proposal.price_per_unit) {
        return Err(Notice::with_args(
            "{0} is outside the range you and Margaret share ({1}). Choose a price in that range.",
            vec![
                NoticeArg::Money(proposal.price_per_unit),
                NoticeArg::Range(shared),
            ],
        ));
    }
    proposal
        .units
        .checked_mul(proposal.price_per_unit)
        .ok_or_else(|| Notice::error("Please enter a whole-dollar price of at least $1."))
}

pub fn match_brief<'a>(
    brief: &Brief,
    experts: impl IntoIterator<Item = &'a Expert>,
) -> MatchResult {
    let mut rejected = Vec::new();
    let mut candidates = Vec::new();
    for expert in experts {
        let Some(price) = expert.price_range.filter(|_| expert.published) else {
            rejected.push(NotMatched {
                expert: expert.clone(),
                reason: NotMatchedReason::Unpublished,
            });
            continue;
        };
        let competencies: Vec<Competency> = expert
            .competencies
            .iter()
            .filter(|competency| competency.tag == brief.competency_tag)
            .cloned()
            .collect();
        if competencies.is_empty() {
            rejected.push(NotMatched {
                expert: expert.clone(),
                reason: NotMatchedReason::Competency,
            });
            continue;
        }
        let Some(shared) = price.overlap(brief.budget) else {
            rejected.push(NotMatched {
                expert: expert.clone(),
                reason: NotMatchedReason::Price,
            });
            continue;
        };
        candidates.push((expert, competencies, shared));
    }
    candidates.sort_by(|a, b| {
        let width = |range: PriceRange| i128::from(range.max) - i128::from(range.min);
        width(b.2)
            .cmp(&width(a.2))
            .then_with(|| b.0.years_experience.cmp(&a.0.years_experience))
    });
    let mut candidates = candidates.into_iter();
    let Some((expert, matched_competencies, overlap)) = candidates.next() else {
        let reason = if rejected
            .iter()
            .any(|entry| entry.reason == NotMatchedReason::Price)
        {
            NotMatchedReason::Price
        } else {
            NotMatchedReason::Competency
        };
        let explanation = if reason == NotMatchedReason::Price {
            Notice::with_args(
                "Experts with “{0}” charge outside your budget of {1} per 15 minutes. Widening the budget would let Agely match you.",
                vec![
                    NoticeArg::Text(brief.competency_label.clone()),
                    NoticeArg::Range(brief.budget),
                ],
            )
        } else {
            Notice::with_args(
                "No published expert lists “{0}” yet.",
                vec![NoticeArg::Text(brief.competency_label.clone())],
            )
        };
        return MatchResult::None {
            reason,
            explanation,
            others: rejected,
        };
    };
    let first_name = expert
        .name
        .split_whitespace()
        .next()
        .unwrap_or(&expert.name)
        .to_owned();
    let labels = matched_competencies
        .iter()
        .map(|competency| format!("“{}”", competency.label))
        .collect::<Vec<_>>()
        .join(", ");
    let explanation = vec![
        Notice::with_args(
            "{0} lists {1}, which covers what you need: “{2}”.",
            vec![
                NoticeArg::Text(first_name.clone()),
                NoticeArg::Text(labels),
                NoticeArg::Text(brief.competency_label.clone()),
            ],
        ),
        Notice::with_args(
            "Your budget is {0} per 15 minutes and {1} charges {2}. You overlap at {3}.",
            vec![
                NoticeArg::Range(brief.budget),
                NoticeArg::Text(first_name),
                NoticeArg::Range(expert.price_range.unwrap_or(overlap)),
                NoticeArg::Range(overlap),
            ],
        ),
        Notice::with_args(
            "{0} years of experience.",
            vec![NoticeArg::Number(expert.years_experience)],
        ),
    ];
    let mut others: Vec<NotMatched> = candidates
        .map(|(expert, _, _)| NotMatched {
            expert: expert.clone(),
            reason: NotMatchedReason::Ranked,
        })
        .collect();
    others.extend(rejected);
    MatchResult::Match {
        expert: Box::new(expert.clone()),
        matched_competencies,
        overlap,
        explanation,
        others,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdviceCategory {
    Legal,
    Medical,
    Financial,
}

impl AdviceCategory {
    pub fn message(self) -> &'static str {
        match self {
            Self::Legal => {
                "Agely doesn't offer legal advice, so this brief can't be sent as written. Remove the legal question, or speak to a qualified professional."
            }
            Self::Medical => {
                "Agely doesn't offer medical advice, so this brief can't be sent as written. Remove the medical question, or speak to a qualified professional."
            }
            Self::Financial => {
                "Agely doesn't offer financial advice, so this brief can't be sent as written. Remove the financial question, or speak to a qualified professional."
            }
        }
    }
}

// These are the browser demo's representative phrases, not a general multilingual classifier.
static ADVICE_PATTERNS: LazyLock<Vec<(AdviceCategory, Regex)>> = LazyLock::new(|| {
    use AdviceCategory::{Financial, Legal, Medical};
    let patterns = [
        (Legal, r"சட்ட ஆலோசனை|कानूनी सलाह"),
        (
            Medical,
            r"மருத்துவ நோயறிதல்|மருத்துவ ஆலோசனை|चिकित्सा निदान|चिकित्सकीय सलाह",
        ),
        (
            Financial,
            r"நிதி ஆலோசனை|முதலீட்டு ஆலோசனை|वित्तीय सलाह|निवेश सलाह",
        ),
        (
            Legal,
            r"(?-u:\blegal advice\b|\blawyers?\b|\blawsuits?\b|\bsue\b|\bsuing\b)",
        ),
        (
            Medical,
            r"(?-u:\bdiagnos\w*|\bmedications?\b|\bsymptoms?\b|\btreatments?\b)",
        ),
        (
            Financial,
            r"(?-u:\binvestment advice\b|\bwhich stocks\b|\btax advice\b|\bretirement savings\b)",
        ),
        (
            Legal,
            r"conseil juridique|asesoramiento legal|rechtsberatung|consulenza legale|法律建议|法律諮詢|법률 조언|法律相談|juridisk rådgivning|юридична консультація|porada prawna|consultanță juridică|oikeudellinen neuvonta|nasihat hukum|payong legal",
        ),
        (
            Medical,
            r"diagnostic médical|diagnóstico médico|medizinische diagnose|diagnosi medica|医疗诊断|醫療診斷|의료 진단|医療診断|medicinsk diagnose|медичний діагноз|diagnoza medyczna|diagnostic medical|medisinsk diagnose|lääketieteellinen diagnoosi|diagnosis medis|medikal na diagnosis",
        ),
        (
            Financial,
            r"conseil fiscal|asesoramiento financiero|anlageberatung|consulenza finanziaria|投资建议|投資建議|투자 조언|投資助言|investeringsrådgivning|фінансова консультація|porada finansowa|consultanță financiară|finansiell rådgivning|sijoitusneuvonta|nasihat investasi|payong pinansyal",
        ),
    ];
    patterns
        .into_iter()
        .map(|(category, pattern)| {
            // Every pattern is a fixed source literal, covered by the classifier table test.
            #[expect(
                clippy::expect_used,
                reason = "static advice regex patterns are validated by the classifier tests"
            )]
            let regex = Regex::new(&format!("(?i){pattern}")).expect("valid static advice pattern");
            (category, regex)
        })
        .collect()
});

pub fn detect_restricted_advice(text: &str) -> Option<AdviceCategory> {
    let normalized = text.nfc().collect::<String>();
    ADVICE_PATTERNS
        .iter()
        .find_map(|(category, pattern)| pattern.is_match(&normalized).then_some(*category))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed() -> Seed {
        let competency = |id: &str, tag: &str| Competency {
            id: id.into(),
            tag: tag.into(),
            label: tag.into(),
            description: String::new(),
            source: CompetencySource::Interview { turn: 2 },
        };
        let margaret = Expert {
            id: "margaret".into(),
            name: "Margaret Okafor".into(),
            age: Some(68),
            headline: String::new(),
            years_experience: 34,
            location: String::new(),
            competencies: vec![
                competency("c1", "contract-manufacturing"),
                competency("c2", "negotiation"),
                competency("c3", "audits"),
                competency("c4", "cost-down"),
            ],
            example_problems: Vec::new(),
            price_range: None,
            published: false,
            credentials_self_declared: true,
        };
        let conversation = vec![
            ConversationTurn {
                speaker: Speaker::Agent,
                text: "Hello".into(),
            },
            ConversationTurn {
                speaker: Speaker::Margaret,
                text: "Procurement".into(),
            },
        ];
        Seed {
            interview_turns: conversation.clone(),
            margaret,
            price_suggestion: PriceSuggestion {
                range: PriceRange { min: 60, max: 90 },
                reasons: Vec::new(),
            },
            profile_replies: vec![SuggestedReply {
                id: "remove".into(),
                text: "Remove cost-down".into(),
                ack: "Removed".into(),
                effect: ReplyEffect::RemoveCompetency { id: "c4".into() },
            }],
            price_replies: vec![SuggestedReply {
                id: "higher".into(),
                text: "Higher".into(),
                ack: "Changed".into(),
                effect: ReplyEffect::SetPrice {
                    range: PriceRange { min: 65, max: 95 },
                },
            }],
            brief_turns: conversation.clone(),
            brief: Brief {
                problem: "Review manufacturing commercial terms".into(),
                competency_tag: "contract-manufacturing".into(),
                competency_label: "Contract manufacturing".into(),
                units: 2,
                budget: PriceRange { min: 50, max: 100 },
                confirmed: false,
            },
            other_experts: Vec::new(),
            debrief_turns: conversation,
            workflow: Workflow {
                name: "Review agreement".into(),
                questions: vec!["Who owns the tooling?".into()],
                checks: Vec::new(),
                watch_outs: Vec::new(),
                status: WorkflowStatus::Proposed,
            },
        }
    }

    fn reveal(state: &mut DemoState) {
        while !state.can_advance() {
            state.dispatch(Action::ShowNextTurn);
        }
    }

    #[test]
    fn the_full_story_requires_explicit_decisions_and_derives_earnings() {
        let mut state = DemoState::new(seed());
        state.dispatch(Action::Next);
        state.dispatch(Action::Next);
        assert_eq!(state.step, 1);
        reveal(&mut state);
        state.dispatch(Action::Next);
        assert_eq!(state.step, 2);
        assert!(!state.can_advance());
        state.dispatch(Action::PublishProfile);
        state.dispatch(Action::Next);
        assert!(!state.can_advance());
        state.dispatch(Action::ConfirmPrice);
        state.dispatch(Action::Next);
        reveal(&mut state);
        state.dispatch(Action::Next);
        assert!(!state.can_advance());
        state.dispatch(Action::ConfirmBrief);
        state.dispatch(Action::Next);
        assert!(matches!(state.current_match(), MatchResult::Match { .. }));
        state.dispatch(Action::Next);
        assert_eq!(state.step, 7);
        assert!(state.session.is_none());
        state.dispatch(Action::SendProposal);
        assert!(state.session.is_none());
        state.dispatch(Action::Next);
        assert!(!state.can_advance());
        state.dispatch(Action::AcceptProposal);
        state.dispatch(Action::Next);
        assert_eq!(state.derive_engine().earnings, 0);
        state.dispatch(Action::MarkSessionHeld);
        assert_eq!(state.derive_engine().earnings, 150);
        state.dispatch(Action::Next);
        reveal(&mut state);
        state.dispatch(Action::Next);
        assert!(state.derive_engine().workflows.is_empty());
        state.dispatch(Action::ApproveWorkflow);
        state.dispatch(Action::Next);
        assert_eq!(state.step, 12);
        assert_eq!(state.derive_engine().sessions_completed, 1);
        assert_eq!(state.derive_engine().workflows.len(), 1);
        assert!(!state.can_advance());
    }

    #[test]
    fn editing_a_published_profile_requires_republication_and_nonempty_existing_skills() {
        let mut state = DemoState::new(seed());
        state.dispatch(Action::PublishProfile);
        state.dispatch(Action::EditCompetency {
            id: "c1".into(),
            label: "Negotiating manufacturing terms".into(),
        });
        assert!(!state.margaret.published);
        assert_eq!(state.notice.as_ref().unwrap().kind, NoticeKind::Info);
        let competencies = state.margaret.competencies.clone();
        for (id, label) in [("c1", "  "), ("missing", "Something")] {
            state.dispatch(Action::EditCompetency {
                id: id.into(),
                label: label.into(),
            });
            assert_eq!(state.margaret.competencies, competencies);
            assert_eq!(state.notice.as_ref().unwrap().kind, NoticeKind::Error);
        }
        state.dispatch(Action::RemoveCompetency { id: "c4".into() });
        state.dispatch(Action::RemoveCompetency { id: "c3".into() });
        state.dispatch(Action::PublishProfile);
        assert!(!state.margaret.published);
    }

    #[test]
    fn suggested_replies_apply_the_same_edits_and_append_the_exchange() {
        let mut state = DemoState::new(seed());
        state.dispatch(Action::ReplyToAgent {
            reply_id: "remove".into(),
        });
        assert_eq!(state.margaret.competencies.len(), 3);
        assert_eq!(state.exchanges[0].len(), 2);
        state.dispatch(Action::ReplyToAgent {
            reply_id: "higher".into(),
        });
        assert_eq!(state.price_draft, PriceRange { min: 65, max: 95 });
        assert!(!state.price_confirmed);
        assert_eq!(state.exchanges[1].len(), 2);
        let before = state.clone();
        state.dispatch(Action::ReplyToAgent {
            reply_id: "unknown".into(),
        });
        assert_eq!(state, before);
    }

    #[test]
    fn invalid_price_edits_preserve_the_previous_price_and_valid_edits_need_confirmation() {
        let mut state = DemoState::new(seed());
        state.dispatch(Action::ConfirmPrice);
        for range in [
            PriceRange { min: 0, max: 90 },
            PriceRange { min: 100, max: 90 },
        ] {
            state.dispatch(Action::SetPrice { range });
            assert_eq!(state.price_draft, PriceRange { min: 60, max: 90 });
            assert!(state.price_confirmed);
            assert_eq!(state.notice.as_ref().unwrap().kind, NoticeKind::Error);
        }
        state.dispatch(Action::SetPrice {
            range: PriceRange { min: 70, max: 95 },
        });
        assert!(!state.price_confirmed);
        assert!(state.margaret.price_range.is_none());
    }

    #[test]
    fn matching_ranks_overlap_before_experience_and_explains_rejections() {
        let mut state = DemoState::new(seed());
        state.dispatch(Action::PublishProfile);
        state.dispatch(Action::ConfirmPrice);
        let mut narrow = state.margaret.clone();
        narrow.id = "narrow".into();
        narrow.years_experience = 90;
        narrow.price_range = Some(PriceRange { min: 70, max: 80 });
        let mut outside = state.margaret.clone();
        outside.id = "outside".into();
        outside.price_range = Some(PriceRange { min: 120, max: 160 });
        let mut wrong_skill = state.margaret.clone();
        wrong_skill.id = "wrong".into();
        wrong_skill.competencies.clear();
        state.seed.other_experts = vec![narrow, outside, wrong_skill];
        let MatchResult::Match {
            expert,
            overlap,
            explanation,
            others,
            ..
        } = state.current_match()
        else {
            panic!("expected match");
        };
        assert_eq!(expert.id, "margaret");
        assert_eq!(overlap, PriceRange { min: 60, max: 90 });
        assert_eq!(explanation.len(), 3);
        assert_eq!(
            others.iter().map(|entry| entry.reason).collect::<Vec<_>>(),
            vec![
                NotMatchedReason::Ranked,
                NotMatchedReason::Price,
                NotMatchedReason::Competency
            ]
        );
        state.seed.other_experts[0].price_range = Some(overlap);
        let MatchResult::Match { expert, .. } = state.current_match() else {
            panic!("expected match");
        };
        assert_eq!(expert.id, "narrow");
    }

    #[test]
    fn no_match_distinguishes_a_missing_skill_from_a_price_gap() {
        let mut state = DemoState::new(seed());
        assert!(matches!(
            state.current_match(),
            MatchResult::None {
                reason: NotMatchedReason::Competency,
                ..
            }
        ));
        state.dispatch(Action::PublishProfile);
        state.dispatch(Action::ConfirmPrice);
        state.brief.budget = PriceRange { min: 1, max: 20 };
        assert!(matches!(
            state.current_match(),
            MatchResult::None {
                reason: NotMatchedReason::Price,
                ..
            }
        ));
    }

    #[test]
    fn brief_confirmation_rejects_invalid_values_and_restricted_advice() {
        for (problem, units, budget) in [
            (" ", 2, PriceRange { min: 50, max: 100 }),
            ("Review a supplier", 0, PriceRange { min: 50, max: 100 }),
            ("Review a supplier", 5, PriceRange { min: 50, max: 100 }),
            ("Review a supplier", 2, PriceRange { min: 90, max: 60 }),
            ("I need legal advice", 2, PriceRange { min: 50, max: 100 }),
        ] {
            let mut state = DemoState::new(seed());
            state.dispatch(Action::EditBrief {
                problem: problem.into(),
                units,
                budget,
            });
            state.dispatch(Action::ConfirmBrief);
            assert!(!state.brief.confirmed, "{problem}");
            assert!(state.notice.is_some());
        }
        let mut state = DemoState::new(seed());
        state.dispatch(Action::ConfirmBrief);
        assert!(state.brief.confirmed);
        state.dispatch(Action::EditBrief {
            problem: "Review tooling ownership".into(),
            units: 3,
            budget: PriceRange { min: 50, max: 100 },
        });
        assert!(!state.brief.confirmed);
        state.dispatch(Action::ConfirmBrief);
        assert_eq!(state.proposal.units, 3);
    }

    #[test]
    fn representative_advice_phrases_use_word_boundaries_and_unicode_normalization() {
        for (text, expected) in [
            ("Need a LAWYER", AdviceCategory::Legal),
            ("ñlawyer", AdviceCategory::Legal),
            ("Discuss my symptoms", AdviceCategory::Medical),
            ("बsymptoms", AdviceCategory::Medical),
            ("Which stocks should I buy?", AdviceCategory::Financial),
            ("சட்ட ஆலோசனை", AdviceCategory::Legal),
            ("चिकित्सा निदान", AdviceCategory::Medical),
            ("consultanță juridică", AdviceCategory::Legal),
            ("conseil fiscal", AdviceCategory::Financial),
            ("医疗诊断", AdviceCategory::Medical),
        ] {
            assert_eq!(detect_restricted_advice(text), Some(expected), "{text}");
            assert_eq!(
                detect_restricted_advice(&text.nfd().collect::<String>()),
                Some(expected),
                "NFD {text}"
            );
        }
        assert_eq!(
            detect_restricted_advice("Review the commercial manufacturing terms"),
            None
        );
        assert_eq!(
            detect_restricted_advice("Susan bought a supplier checklist"),
            None
        );
    }

    #[test]
    fn booking_and_payment_release_are_explicit_and_idempotent() {
        let mut state = DemoState::new(seed());
        state.jump(4);
        assert!(state.session.is_none());
        state.dispatch(Action::AcceptProposal);
        assert!(state.session.is_none());
        state.dispatch(Action::SendProposal);
        assert!(state.session.is_none());
        state.dispatch(Action::AcceptProposal);
        let session = state.session.clone().unwrap();
        assert_eq!(session.total, 150);
        assert_eq!(session.payment.status, PaymentStatus::Held);
        assert!(session.payment.demo && session.call.is_demo);
        state.dispatch(Action::AcceptProposal);
        state.dispatch(Action::DeclineProposal);
        assert_eq!(state.session, Some(session));
        state.dispatch(Action::MarkSessionHeld);
        let settled = state.session.clone();
        state.dispatch(Action::MarkSessionHeld);
        assert_eq!(state.session, settled);
        assert_eq!(state.derive_engine().earnings, 150);
    }

    #[test]
    fn declining_returns_to_the_proposal_without_creating_a_session() {
        let mut state = DemoState::new(seed());
        state.jump(4);
        state.dispatch(Action::SendProposal);
        state.dispatch(Action::Next);
        state.dispatch(Action::DeclineProposal);
        assert_eq!(state.step, 7);
        assert!(!state.proposal.sent);
        assert!(state.session.is_none());
        state.dispatch(Action::SendProposal);
        state.dispatch(Action::AcceptProposal);
        assert!(state.session.is_some());
    }

    #[test]
    fn invalid_or_overflowing_proposals_cannot_be_sent_or_accepted() {
        let mut state = DemoState::new(seed());
        state.jump(4);
        for price_per_unit in [0, -1, 59, 91] {
            state.dispatch(Action::SetProposalPrice { price_per_unit });
            state.dispatch(Action::SendProposal);
            assert!(!state.proposal.sent);
            assert!(state.session.is_none());
        }
        let enormous = Proposal {
            units: 4,
            price_per_unit: i64::MAX,
            sent: false,
        };
        assert!(
            validate_proposal(
                &enormous,
                PriceRange {
                    min: 1,
                    max: i64::MAX
                }
            )
            .is_err()
        );
        for units in [0, 5, i64::MAX] {
            state.dispatch(Action::SetProposalUnits { units });
            assert_eq!(state.proposal.units, 2);
        }
    }

    #[test]
    fn chapter_jumps_replay_defaults_and_restart_restores_the_seed() {
        let mut state = DemoState::new(seed());
        state.jump(6);
        assert_eq!(state.step, 12);
        assert_eq!(state.derive_engine().earnings, 150);
        assert_eq!(state.derive_engine().workflows.len(), 1);
        state.jump(3);
        assert_eq!(state.step, 6);
        assert!(state.brief.confirmed && state.price_confirmed && state.margaret.published);
        assert!(state.session.is_none());
        let before = state.clone();
        state.jump(99);
        assert_eq!(state, before);
        state.dispatch(Action::Restart);
        assert_eq!(state, DemoState::new(seed()));
    }

    #[test]
    fn bundled_story_deserializes_and_completes_in_every_supported_language() {
        let english = crate::i18n::I18n::new("en").unwrap();
        for language in english.locales() {
            let seed: Seed = english.with_locale(&language.id).localized_seed().unwrap();
            let mut state = DemoState::new(seed);
            state.jump(6);
            let engine = state.derive_engine();
            assert_eq!(state.step, 12, "{}", language.id);
            assert_eq!(engine.sessions_completed, 1, "{}", language.id);
            assert_eq!(engine.earnings, 150, "{}", language.id);
            assert_eq!(engine.workflows.len(), 1, "{}", language.id);
        }
    }
}
