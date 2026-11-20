use std::borrow::Cow;

use agely_native::{
    app::{Agely, DismissLanguages, PreviousTab, Quit, Tab},
    i18n::{I18n, load_locale},
    input,
};
use gpui::{
    App, AppContext, Bounds, KeyBinding, Menu, MenuItem, TitlebarOptions, WindowBounds,
    WindowOptions, px, size,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("Could not start Agely: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let i18n = I18n::new(&load_locale())?;
    let seed = i18n.localized_seed()?;
    gpui_platform::application().run(move |cx: &mut App| {
        if let Err(error) = cx.text_system().add_fonts(vec![Cow::Borrowed(
            include_bytes!("../assets/fonts/Fraunces-600.ttf").as_slice(),
        )]) {
            eprintln!("Could not register the bundled display font: {error}");
        }
        input::init(cx);
        cx.bind_keys([
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("tab", Tab, None),
            KeyBinding::new("shift-tab", PreviousTab, None),
            KeyBinding::new("escape", DismissLanguages, None),
        ]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.set_menus([Menu::new("Agely").items([MenuItem::action("Quit Agely", Quit)])]);
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        let bounds = Bounds::centered(None, size(px(1280.0), px(832.0)), cx);
        let window = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(960.0), px(640.0))),
                titlebar: Some(TitlebarOptions {
                    title: Some(format!("Agely v{}", agely_native::VERSION).into()),
                    ..Default::default()
                }),
                app_id: Some("com.agely.app".into()),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| Agely::new(i18n, seed, window, cx)),
        );
        if let Err(error) = window {
            eprintln!("Could not open the Agely window: {error}");
            cx.quit();
        }
        cx.activate(true);
    });
    Ok(())
}
