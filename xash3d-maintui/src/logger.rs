use xash3d_ui::engine;

pub fn init() {
    let dev = engine().cvar::<f32>(c"developer") as i32;
    xash3d_ui::utils::logger::init(dev, |s| {
        engine().con_print(s);
    });
}
