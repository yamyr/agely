//! Shared native reading and control surfaces.

use crate::theme::theme;
use gpui::{
    AnyElement, Bounds, ElementId, FocusHandle, FontWeight, Pixels, Role, ScrollHandle,
    SharedString, Window, div, point, prelude::*, px, relative,
};
use std::{cell::Cell, rc::Rc};

#[derive(Clone, Copy)]
struct FocusLayout {
    control: Bounds<Pixels>,
    viewport: Bounds<Pixels>,
}

#[derive(Clone)]
pub(crate) struct ControlFocus {
    pub handle: FocusHandle,
    was_focused: Rc<Cell<bool>>,
    layout: Rc<Cell<Option<FocusLayout>>>,
}

impl ControlFocus {
    pub fn new(handle: FocusHandle) -> Self {
        Self {
            handle,
            was_focused: Rc::new(Cell::new(false)),
            layout: Rc::default(),
        }
    }

    pub fn reset_unfocused(&self, window: &Window) {
        if !self.handle.is_focused(window) {
            self.was_focused.set(false);
            self.layout.set(None);
        }
    }

    pub fn reveal(&self, control: impl IntoElement, viewport: &ScrollHandle) -> AnyElement {
        let focus = self.clone();
        let viewport = viewport.clone();
        div()
            .min_w_0()
            .max_w_full()
            .child(control)
            .on_children_prepainted(move |children, window, cx| {
                if !focus.handle.is_focused(window) {
                    focus.reset_unfocused(window);
                    return;
                }
                let Some(control) = children.first() else {
                    return;
                };
                let bounds = viewport.bounds();
                let offset = viewport.offset();
                let max = viewport.max_offset();
                let layout = FocusLayout {
                    control: *control - offset,
                    viewport: bounds,
                };
                // Subtract scrolling so a person can scroll away while focus stays put.
                // Ignore subpixel arithmetic noise when comparing layout coordinates.
                let same_bounds = |a: Bounds<Pixels>, b: Bounds<Pixels>| {
                    (a.left() - b.left()).abs() < px(0.01)
                        && (a.top() - b.top()).abs() < px(0.01)
                        && (a.size.width - b.size.width).abs() < px(0.01)
                        && (a.size.height - b.size.height).abs() < px(0.01)
                };
                let unchanged = focus.layout.replace(Some(layout)).is_some_and(|previous| {
                    same_bounds(previous.control, layout.control)
                        && same_bounds(previous.viewport, layout.viewport)
                });
                if focus.was_focused.replace(true) && unchanged {
                    return;
                }
                let reveal = |start: Pixels,
                              end: Pixels,
                              view_start: Pixels,
                              view_end: Pixels,
                              offset: Pixels,
                              max: Pixels| {
                    let delta = if start < view_start {
                        view_start - start
                    } else if end > view_end {
                        view_end - end
                    } else {
                        px(0.0)
                    };
                    (offset + delta).clamp(-max, px(0.0))
                };
                let next = point(
                    reveal(
                        control.left(),
                        control.right(),
                        bounds.left(),
                        bounds.right(),
                        offset.x,
                        max.x,
                    ),
                    reveal(
                        control.top(),
                        control.bottom(),
                        bounds.top(),
                        bounds.bottom(),
                        offset.y,
                        max.y,
                    ),
                );
                if next != offset {
                    viewport.set_offset(next);
                    // refresh() is ignored during prepaint; schedule the corrected position
                    // for a physical frame rather than waiting for the next user input.
                    window.defer(cx, |window, _| window.refresh());
                }
            })
            .into_any_element()
    }
}

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
