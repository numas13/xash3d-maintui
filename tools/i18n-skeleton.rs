use std::{
    collections::HashSet,
    env,
    sync::{LazyLock, Mutex},
};

struct State {
    strip: bool,
    map: HashSet<&'static str>,
}

static STATE: LazyLock<Mutex<State>> = LazyLock::new(|| {
    Mutex::new(State {
        strip: env::args().nth(1).is_some_and(|i| i.starts_with("strip")),
        map: HashSet::new(),
    })
});

fn print(mut i: &'static str) {
    let mut state = STATE.lock().unwrap();
    if let Some(s) = i.strip_prefix('#') {
        if state.strip {
            return;
        }
        i = s;
    }
    if !i.is_empty() && !state.map.contains(i) {
        state.map.insert(i);
        let pad = 32_usize.saturating_sub(i.len());
        println!("\"{i}\"{:pad$}\"\"", ' ');
    }
}
fn main() {
    println!("// How to work with this file:");
    println!(
        "// 1) This file must be called maintui_x.txt, where x is your language in lower case"
    );
    println!("// 2) Tokens on left are original strings, on right is translation");
    println!("// 3) .txt version of this file is intended to be used if you");
    println!("// don't want to rely on strings.lst nor on non-free translations");
    println!("// 4) _stripped.txt version of this file is intended to be used if you");
    println!("// WANT to rely on original game files(both strings.lst and non-free translations)");
    println!("\"lang\"");
    println!("{{");
    println!("\"Language\" \"<YOUR_LANGUAGE_HERE>\"");
    println!("\"Tokens\"");
    println!("{{");

    macro_rules! define_strings {
        ($($name:ident { $($body:tt)* })*) => ({
            $(
                define_strings!($($body)*);
            )*
        });
        ($($name:ident = $value:expr),* $(,)?) => {
            $(print($value);)*
        };
    }
    include!("../xash3d-maintui/src/i18n.rs");

    println!("}}");
    println!("}}");
}
