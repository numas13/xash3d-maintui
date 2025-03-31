use res::valve::sound;
use xash3d_ui::engine;

pub fn select_changed() {
    engine().play_sound(sound::common::LAUNCH_GLOW1);
}

pub fn select_item() {
    engine().play_sound(sound::common::LAUNCH_SELECT2);
}

pub fn switch_menu() {
    engine().play_sound(sound::common::LAUNCH_SELECT2);
}

pub fn deny() {
    engine().play_sound(sound::common::LAUNCH_SELECT1);
}

pub fn deny2() {
    engine().play_sound(sound::common::LAUNCH_DENY2);
}

pub fn confirm() {
    engine().play_sound(sound::common::LAUNCH_SELECT2);
}

pub fn select_prev() {
    engine().play_sound(sound::common::LAUNCH_UPMENU1);
}

pub fn select_next() {
    engine().play_sound(sound::common::LAUNCH_DNMENU1);
}
