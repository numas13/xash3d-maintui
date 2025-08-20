use std::{
    cell::{Ref, RefCell, RefMut},
    ffi::c_int,
};

use csz::CStrThin;
use xash3d_ui::{
    color::RGBA,
    engine,
    export::{self, impl_unsync_global, UiDll},
    raw::{self, netadr_s},
};

use crate::ui::Ui;

#[derive(Default)]
pub struct Instance {
    ui: RefCell<Ui>,
}

impl_unsync_global!(Instance);

impl Instance {
    pub fn ui_ref(&self) -> Ref<'_, Ui> {
        self.ui.borrow()
    }

    pub fn ui_mut(&self) -> RefMut<'_, Ui> {
        self.ui.borrow_mut()
    }
}

impl UiDll for Instance {
    fn vid_init(&self) -> bool {
        self.ui_mut().vid_init()
    }

    fn redraw(&self, time: f32) {
        self.ui_mut().redraw(time);
    }

    fn key_event(&self, key: c_int, down: bool) {
        self.ui_mut().key_event(key, down);
    }

    fn mouse_move(&self, x: c_int, y: c_int) {
        self.ui_mut().mouse_move(x, y);
    }

    fn set_active_menu(&self, active: bool) {
        engine().key_clear_states();
        self.ui_mut().set_active_menu(active);
    }

    fn add_server_to_list(&self, addr: netadr_s, info: &CStrThin) {
        self.ui_mut()
            .add_server_to_list(addr, &info.as_c_str().to_string_lossy());
    }

    fn is_visible(&self) -> bool {
        self.ui_ref().is_visible()
    }

    fn add_touch_button_to_list(
        &self,
        name: &CStrThin,
        texture: &CStrThin,
        command: &CStrThin,
        color: RGBA,
        flags: c_int,
    ) {
        self.ui_mut()
            .add_touch_button_to_list(name, texture, command, color, flags);
    }

    fn reset_ping(&self) {
        self.ui_mut().reset_ping();
    }
}

#[no_mangle]
pub extern "C" fn GetMenuAPI(
    ret: Option<&mut raw::UI_FUNCTIONS>,
    funcs: Option<&raw::ui_enginefuncs_s>,
    globals: Option<&'static raw::ui_globalvars_s>,
) -> c_int {
    std::panic::set_hook(Box::new(|info| {
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| info.payload().downcast_ref::<String>().map(|s| s.as_str()))
            .unwrap_or("<failed to print panic payload>");

        if let Some(loc) = info.location() {
            let file = loc.file();
            let line = loc.line();
            let col = loc.column();
            error!("maintui panicked at {file}:{line}:{col}:\n{payload}");
        } else {
            error!("maintui panicked:\n{payload}");
        }
    }));

    export::get_menu_api::<Instance>(ret, funcs, globals)
}

#[no_mangle]
pub extern "C" fn GetExtAPI(
    version: c_int,
    ret: Option<&mut raw::UI_EXTENDED_FUNCTIONS>,
    funcs: Option<&raw::ui_extendedfuncs_s>,
) -> c_int {
    export::get_ext_api::<Instance>(version, ret, funcs)
}
