//! Shared native reading and control surfaces.

use crate::theme::theme;
use gpui::{AnyElement, ElementId, FontWeight, Role, SharedString, div, prelude::*, px, relative};

pub fn copy(id: impl Into<ElementId>, text: impl Into<SharedString>) -> AnyElement {
    let text = text.into();
    div()
        .id(id)
        .role(Role::Label)
        .aria_label(text.clone())
        .w_full()
        .line_height(relative(1.5))
        .child(text)
        .into_any_element()
}

pub fn muted(id: impl Into<ElementId>, text: impl Into<SharedString>) -> AnyElement {
    div()
        .text_color(theme().muted)
        .child(copy(id, text))
        .into_any_element()
}

pub fn heading(
    id: impl Into<ElementId>,
    text: impl Into<SharedString>,
    level: usize,
) -> AnyElement {
    let text = text.into();
    div()
        .id(id)
        .role(Role::Heading)
        .aria_level(level)
        .aria_label(text.clone())
        .font_family("Fraunces")
        .font_weight(FontWeight::SEMIBOLD)
        .text_size(px(if level == 2 {
            theme().title
        } else {
            theme().section
        }))
        .line_height(relative(1.2))
        .child(text)
        .into_any_element()
}

pub fn column(children: Vec<AnyElement>) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(theme().space))
        .w_full()
        .min_w_0()
        .children(children)
        .into_any_element()
}

pub fn row(children: Vec<AnyElement>) -> AnyElement {
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(px(theme().space_small))
        .children(children)
        .into_any_element()
}

pub fn card(id: impl Into<ElementId>, children: Vec<AnyElement>) -> AnyElement {
    div()
        .id(id)
        .w_full()
        .min_w_0()
        .p(px(theme().space_large))
        .bg(theme().surface)
        .border_1()
        .border_color(theme().border)
        .rounded(px(theme().radius))
        .child(column(children))
        .into_any_element()
}

pub fn notice(id: impl Into<ElementId>, text: impl Into<SharedString>, error: bool) -> AnyElement {
    let text = text.into();
    div()
        .id(id)
        .role(if error { Role::Alert } else { Role::Status })
        .aria_label(text.clone())
        .w_full()
        .p(px(theme().space))
        .rounded(px(theme().radius))
        .bg(if error {
            theme().error_bg
        } else {
            theme().soft
        })
        .text_color(if error {
            theme().error_text
        } else {
            theme().text
        })
        .line_height(relative(1.5))
        .child(text)
        .into_any_element()
}
