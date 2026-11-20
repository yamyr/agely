use gpui::{
    EntityInputHandler, KeyBinding, KeyDownEvent, KeyUpEvent, Keystroke, TestAppContext,
    VisualTestContext,
};

use super::*;

fn ready() -> AgentStatus {
    AgentStatus {
        state: StatusState::Ready,
        model: Some("test-model".into()),
        detail: String::new(),
        fix: None,
    }
}

fn setup(cx: &mut TestAppContext, step: usize) -> (Entity<Agely>, &mut VisualTestContext) {
    cx.update(|cx| {
        crate::input::init(cx);
        cx.bind_keys([
            KeyBinding::new("tab", Tab, None),
            KeyBinding::new("shift-tab", PreviousTab, None),
            KeyBinding::new("escape", DismissLanguages, None),
        ]);
    });
    cx.add_window_view(move |window, cx| {
        let i18n = I18n::new("en").unwrap();
        let seed = i18n.localized_seed().unwrap();
        let mut app = Agely::create(i18n, seed, window, cx);
        if let Some(chapter) = CHAPTER_STEPS.iter().rposition(|start| *start <= step) {
            app.state.jump(chapter);
        }
        app.state.step = step;
        app
    })
}

fn press_key(cx: &mut VisualTestContext, key: &'static str) {
    let keystroke = Keystroke::parse(key).unwrap();
    cx.simulate_event(KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(KeyUpEvent { keystroke });
}

#[gpui::test]
fn enter_and_space_activate_a_button_once(cx: &mut TestAppContext) {
    let (app, cx) = setup(cx, 0);
    press_key(cx, "tab");
    app.update_in(cx, |app, window, _| {
        assert!(
            app.language_menu_focus.is_focused(window),
            "first Tab must focus Language"
        )
    });

    press_key(cx, "enter");
    app.read_with(cx, |app, _| assert!(app.languages_open));
    press_key(cx, "enter");
    app.read_with(cx, |app, _| assert!(!app.languages_open));

    press_key(cx, "space");
    app.read_with(cx, |app, _| assert!(app.languages_open));
    press_key(cx, "escape");
    app.read_with(cx, |app, _| assert!(!app.languages_open));
}

#[gpui::test]
fn language_confirmation_keeps_keyboard_actions_inside_the_dialog(cx: &mut TestAppContext) {
    let (app, cx) = setup(cx, 2);
    let (skill_id, original_name) = app.read_with(cx, |app, _| {
        let skill = &app.state.margaret.competencies[0];
        (skill.id.clone(), skill.label.clone())
    });
    let field_id = format!("skill-{skill_id}");
    app.update_in(cx, |app, window, cx| {
        let field = &app.fields[&field_id];
        let focus = field.read(cx).focus_handle.clone();
        field.update(cx, |field, cx| {
            field.set_value("Unsubmitted profile edit", cx)
        });
        window.focus(&focus, cx);
    });
    // Leave the underlying Save name control focused before opening the dialog.
    press_key(cx, "tab");
    app.update_in(cx, |app, window, cx| {
        app.handle(UiAction::ChooseLanguage("ja".into()), window, cx);
        assert!(app.language_cancel_focus.is_focused(window));
    });

    // The dialog has two controls; a full focus cycle must return to Keep current language.
    press_key(cx, "tab");
    press_key(cx, "tab");
    app.update_in(cx, |app, window, _| {
        assert!(
            app.language_cancel_focus.is_focused(window),
            "two dialog tabs must return to cancel"
        )
    });
    press_key(cx, "enter");
    app.read_with(cx, |app, cx| {
        assert!(app.language_confirmation.is_none());
        assert_eq!(app.i18n.locale(), "en");
        assert_eq!(app.state.step, 2);
        let skill = app
            .state
            .margaret
            .competencies
            .iter()
            .find(|skill| skill.id == skill_id)
            .unwrap();
        assert_eq!(skill.label, original_name);
        assert_eq!(app.field_value(&field_id, cx), "Unsubmitted profile edit");
    });

    press_key(cx, "space");
    app.read_with(cx, |app, _| assert!(app.languages_open));
}

#[gpui::test]
fn selecting_a_confirmed_brief_does_not_invalidate_it(cx: &mut TestAppContext) {
    let (app, cx) = setup(cx, 5);
    app.update_in(cx, |app, window, cx| {
        app.state.brief.confirmed = true;
        let field = &app.fields["brief-problem"];
        let focus = field.read(cx).focus_handle.clone();
        window.focus(&focus, cx);
    });

    cx.simulate_keystrokes("right shift-right");
    app.read_with(cx, |app, _| assert!(app.state.brief.confirmed));

    cx.simulate_input("new problem");
    app.read_with(cx, |app, cx| {
        assert!(!app.state.brief.confirmed);
        assert_eq!(
            app.state.brief.problem,
            app.field_value("brief-problem", cx)
        );
        assert!(app.state.brief.problem.contains("new problem"));
    });
}

#[gpui::test]
fn selecting_a_sent_proposal_does_not_invalidate_it(cx: &mut TestAppContext) {
    let (app, cx) = setup(cx, 7);
    app.update_in(cx, |app, window, cx| {
        app.state.proposal.sent = true;
        let field = &app.fields["proposal-price"];
        let focus = field.read(cx).focus_handle.clone();
        window.focus(&focus, cx);
    });

    cx.simulate_keystrokes("right shift-right");
    app.read_with(cx, |app, _| assert!(app.state.proposal.sent));

    cx.simulate_input("9");
    app.read_with(cx, |app, _| assert!(!app.state.proposal.sent));
}

#[gpui::test]
fn budget_edits_survive_back_and_next_without_confirming(cx: &mut TestAppContext) {
    let (app, cx) = setup(cx, 5);
    app.update_in(cx, |app, _, cx| {
        app.state.shown[1] = app.state.conversation_turns(1).len();
        for (id, value) in [("budget-min", "70"), ("budget-max", "95")] {
            app.fields[id]
                .clone()
                .update(cx, |field, cx| field.set_value(value, cx));
        }
    });
    cx.run_until_parked();

    app.update_in(cx, |app, window, cx| {
        app.handle(UiAction::Domain(Action::Back), window, cx);
        assert_eq!(app.state.step, 4);
        app.handle(UiAction::Domain(Action::Next), window, cx);
        assert_eq!(app.state.step, 5);
    });
    cx.run_until_parked();

    app.read_with(cx, |app, cx| {
        assert_eq!(
            app.state.brief.budget,
            crate::model::PriceRange { min: 70, max: 95 }
        );
        assert_eq!(app.field_value("budget-min", cx), "70");
        assert_eq!(app.field_value("budget-max", cx), "95");
        assert!(!app.state.brief.confirmed);
    });
}

#[gpui::test]
fn proposal_price_edits_survive_back_and_next_without_sending(cx: &mut TestAppContext) {
    let (app, cx) = setup(cx, 7);
    app.update_in(cx, |app, _, cx| {
        app.fields["proposal-price"]
            .clone()
            .update(cx, |field, cx| field.set_value("80", cx));
    });
    cx.run_until_parked();

    app.update_in(cx, |app, window, cx| {
        app.handle(UiAction::Domain(Action::Back), window, cx);
        assert_eq!(app.state.step, 6);
        app.handle(UiAction::Domain(Action::Next), window, cx);
        assert_eq!(app.state.step, 7);
    });
    cx.run_until_parked();

    app.read_with(cx, |app, cx| {
        assert_eq!(app.state.proposal.price_per_unit, 80);
        assert_eq!(app.field_value("proposal-price", cx), "80");
        assert!(!app.state.proposal.sent);
    });
}

#[gpui::test]
fn chapter_replay_preserves_scripted_conversations_when_going_back(cx: &mut TestAppContext) {
    let (app, cx) = setup(cx, 0);
    app.update_in(cx, |app, window, cx| {
        app.status = Some(ready());
        app.handle(UiAction::Domain(Action::JumpTo { chapter: 1 }), window, cx);
        app.handle(UiAction::Domain(Action::Back), window, cx);
        app.handle(UiAction::Domain(Action::Back), window, cx);
        assert_eq!(app.state.step, 1);
        assert_eq!(app.state.shown[0], app.state.conversation_turns(0).len());
        assert!(!app.is_live(0));
        assert!(app.can_advance());
    });
}

#[gpui::test]
fn finishing_a_failed_live_reply_unblocks_next(cx: &mut TestAppContext) {
    let (app, cx) = setup(cx, 1);
    app.update_in(cx, |app, window, cx| {
        app.status = Some(ready());
        assert!(app.live[0].reply("Procurement", 1));
        app.live[0].fail(1, "Unavailable".into());
        assert!(!app.can_advance());
        app.handle(UiAction::FinishLive(0), window, cx);
        assert!(app.can_advance());
        app.handle(UiAction::Domain(Action::Next), window, cx);
        assert_eq!(app.state.step, 2);
    });
}

#[gpui::test]
fn native_composition_replaces_marked_text_and_commits_unicode(cx: &mut TestAppContext) {
    let (app, cx) = setup(cx, 5);
    app.update_in(cx, |app, window, cx| {
        let field = app.fields["brief-problem"].clone();
        field.update(cx, |input, cx| {
            input.set_value("A🧭B", cx);
            input.replace_and_mark_text_in_range(Some(1..3), "に", Some(1..1), window, cx);
            assert_eq!(input.value(), "AにB");
            assert_eq!(input.marked_text_range(window, cx), Some(1..2));
            assert_eq!(
                input.selected_text_range(false, window, cx).unwrap().range,
                2..2
            );

            input.replace_and_mark_text_in_range(None, "日本🧭", Some(4..4), window, cx);
            assert_eq!(input.value(), "A日本🧭B");
            assert_eq!(input.marked_text_range(window, cx), Some(1..5));
            assert_eq!(
                input.selected_text_range(false, window, cx).unwrap().range,
                5..5
            );
            let mut actual_range = None;
            assert_eq!(
                input
                    .text_for_range(1..5, &mut actual_range, window, cx)
                    .as_deref(),
                Some("日本🧭")
            );
            assert_eq!(actual_range, Some(1..5));

            input.replace_text_in_range(None, "日本語🧭", window, cx);
            assert_eq!(input.value(), "A日本語🧭B");
            assert_eq!(input.marked_text_range(window, cx), None);
            assert_eq!(
                input.selected_text_range(false, window, cx).unwrap().range,
                6..6
            );
            assert_eq!(input.text_length_utf16(window, cx), Some(7));
        });
    });
    app.read_with(cx, |app, _| {
        assert_eq!(app.state.brief.problem, "A日本語🧭B")
    });
}

#[gpui::test]
fn unmarking_native_composition_preserves_text_and_utf16_selection(cx: &mut TestAppContext) {
    let (app, cx) = setup(cx, 5);
    app.update_in(cx, |app, window, cx| {
        let field = app.fields["brief-problem"].clone();
        field.update(cx, |input, cx| {
            input.set_value("前🧭後", cx);
            input.replace_and_mark_text_in_range(Some(1..3), "かな🧭", Some(1..2), window, cx);
            assert_eq!(input.value(), "前かな🧭後");
            assert_eq!(input.marked_text_range(window, cx), Some(1..5));
            assert_eq!(
                input.selected_text_range(false, window, cx).unwrap().range,
                2..3
            );

            input.unmark_text(window, cx);
            assert_eq!(input.value(), "前かな🧭後");
            assert_eq!(input.marked_text_range(window, cx), None);
            assert_eq!(
                input.selected_text_range(false, window, cx).unwrap().range,
                2..3
            );
            input.replace_text_in_range(None, "名", window, cx);
            assert_eq!(input.value(), "前か名🧭後");
            assert_eq!(
                input.selected_text_range(false, window, cx).unwrap().range,
                3..3
            );
        });
    });
    app.read_with(cx, |app, _| {
        assert_eq!(app.state.brief.problem, "前か名🧭後")
    });
}
