mod browser;
mod change_game;
mod config;
mod create_server;
mod main;
mod saves;
mod test;

use alloc::boxed::Box;

macro_rules! define_menu_items {
    ($($name:ident = $text:expr, $hint:expr;)*) => {
        $(const $name: &str = $text;)*

        fn get_menu_hint(item: &str) -> Option<&'static str> {
            let hint = match item {
                $($name => $hint,)*
                _ => return None,
            };
            Some($crate::strings::get(hint))
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
    change_game = change_game::ChangeGame::new(),
}
