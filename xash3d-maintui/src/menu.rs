mod browser;
mod config;
mod create_server;
mod main;
mod saves;
mod test;

macro_rules! define_menu_items {
    ($($name:ident = $text:expr, $hint:expr;)*) => {
        $(const $name: &str = $text;)*

        fn get_menu_hint(item: &str) -> Option<&'static str> {
            let hint = match item {
                $($name => $hint,)*
                _ => "",
            };
            if !hint.is_empty() {
                $crate::strings::try_get(hint)
            } else {
                None
            }
        }
    };
}
pub(crate) use define_menu_items;

macro_rules! define {
    ($($name:ident = $menu:expr),* $(,)?) => {
        $(pub fn $name() -> Box<dyn crate::ui::Menu> {
            Box::new($menu)
        })*
    };
}

define! {
    main = main::MainMenu::new(),
    load = saves::SavesMenu::new(false),
    save = saves::SavesMenu::new(true),
    internet = browser::Browser::new(false),
    lan = browser::Browser::new(true),
    test = test::TestMenu::new(),
    config = config::ConfigMenu::new(),
}
