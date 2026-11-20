//! Request-scoped state for the three optional conversations.

use crate::agent::{AgentTurn, Speaker, TranscriptLine};

#[derive(Clone, Default)]
pub struct Conversation {
    pub lines: Vec<TranscriptLine>,
    pub options: Vec<String>,
    pub pending: Option<u64>,
    pub ended: bool,
    pub scripted: bool,
    pub error: Option<String>,
}

impl Conversation {
    pub fn request(&mut self, request_id: u64) -> bool {
        if self.pending.is_some() || self.ended || self.scripted {
            return false;
        }
        self.pending = Some(request_id);
        self.error = None;
        true
    }

    pub fn reply(&mut self, text: &str, request_id: u64) -> bool {
        if text.trim().is_empty() || self.error.is_some() || !self.request(request_id) {
            return false;
        }
        self.lines.push(TranscriptLine {
            speaker: Speaker::Person,
            text: text.trim().to_owned(),
        });
        self.options.clear();
        true
    }

    pub fn receive(&mut self, request_id: u64, turn: AgentTurn) {
        if self.pending != Some(request_id) {
            return;
        }
        self.pending = None;
        self.lines.push(TranscriptLine {
            speaker: Speaker::Agent,
            text: turn.message,
        });
        self.options = turn.options;
        self.ended = turn.done;
        self.error = None;
    }

    pub fn fail(&mut self, request_id: u64, message: String) {
        if self.pending == Some(request_id) {
            self.pending = None;
            self.error = Some(message);
        }
    }

    pub fn can_finish(&self) -> bool {
        self.pending.is_none()
            && !self.ended
            && self
                .lines
                .iter()
                .any(|line| line.speaker == Speaker::Person)
    }

    pub fn finish(&mut self) {
        if self.can_finish() {
            self.ended = true;
            self.options.clear();
            self.error = None;
        }
    }

    pub fn fallback(&mut self) {
        self.scripted = true;
        self.pending = None;
        self.error = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn() -> AgentTurn {
        AgentTurn {
            message: "What did you do?".into(),
            options: vec!["Procurement".into(), "Sales".into(), "Other".into()],
            done: false,
        }
    }

    #[test]
    fn late_response_cannot_replace_current_request() {
        let mut conversation = Conversation::default();
        assert!(conversation.request(2));
        conversation.receive(1, turn());
        assert!(conversation.lines.is_empty());
        assert_eq!(conversation.pending, Some(2));
        conversation.receive(2, turn());
        assert_eq!(conversation.lines.len(), 1);
        assert_eq!(conversation.options.len(), 3);
    }

    #[test]
    fn pending_or_failed_reply_cannot_send_twice_and_retry_keeps_transcript() {
        let mut conversation = Conversation::default();
        assert!(conversation.reply("Procurement", 1));
        assert!(!conversation.reply("Again", 2));
        conversation.fail(1, "Unavailable".into());
        assert!(!conversation.reply("Again", 3));
        assert!(conversation.request(4));
        conversation.receive(4, turn());
        assert_eq!(conversation.lines.len(), 2);
        assert!(conversation.can_finish());
        conversation.finish();
        assert!(!conversation.reply("Again", 5));
    }

    #[test]
    fn scripted_fallback_ignores_an_in_flight_answer() {
        let mut conversation = Conversation::default();
        assert!(conversation.request(1));
        conversation.fallback();
        conversation.receive(1, turn());
        assert!(conversation.lines.is_empty());
        assert!(conversation.scripted);
    }

    #[test]
    fn finishing_after_a_failed_reply_clears_the_error_gate() {
        let mut conversation = Conversation::default();
        assert!(conversation.reply("Procurement", 1));
        conversation.fail(1, "Unavailable".into());
        assert!(conversation.can_finish());
        conversation.finish();
        assert!(conversation.ended);
        assert!(conversation.error.is_none());
        assert!(conversation.pending.is_none());
    }
}
