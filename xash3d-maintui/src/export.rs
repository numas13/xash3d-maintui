#![allow(clippy::missing_safety_doc)]

use std::{
    cell::{RefCell, RefMut},
    ffi::{c_char, c_int, c_uchar, CStr},
};

use csz::CStrThin;
use xash3d_cell::SyncOnceCell;
use xash3d_shared::raw::netadr_s;
use xash3d_ui::{
    engine,
    raw::{self, MENU_EXTENDED_API_VERSION},
};

use crate::ui::Ui;

pub trait MenuApi<T: UiFunctions> {
    fn global() -> RefMut<'static, Ui>;

    fn ui_funcs() -> raw::UI_FUNCTIONS {
        raw::UI_FUNCTIONS {
            pfnVidInit: Some(Self::vid_init),
            pfnInit: Some(Self::init),
            pfnShutdown: Some(Self::shutdown),
            pfnRedraw: Some(Self::redraw),
            pfnKeyEvent: Some(Self::key_event),
            pfnMouseMove: Some(Self::mouse_move),
            pfnSetActiveMenu: Some(Self::set_active_menu),
            pfnAddServerToList: Some(Self::add_server_to_list),
            pfnGetCursorPos: Some(Self::get_cursor_pos),
            pfnSetCursorPos: Some(Self::set_cursor_pos),
            pfnShowCursor: Some(Self::show_cursor),
            pfnCharEvent: Some(Self::char_event),
            pfnMouseInRect: Some(Self::mouse_in_rect),
            pfnIsVisible: Some(Self::is_visible),
            pfnCreditsActive: Some(Self::credits_active),
            pfnFinalCredits: Some(Self::final_credits),
        }
    }

    unsafe extern "C" fn vid_init() -> c_int {
        Self::global().vid_init()
    }

    unsafe extern "C" fn init() {
        trace!("Ui::init()");

        UI.set(RefCell::new(Ui::new())).ok();
    }

    unsafe extern "C" fn shutdown() {
        Self::global().shutdown()
    }

    unsafe extern "C" fn redraw(time: f32) {
        Self::global().redraw(time)
    }

    unsafe extern "C" fn key_event(key: c_int, down: c_int) {
        Self::global().key_event(key, down)
    }

    unsafe extern "C" fn mouse_move(x: c_int, y: c_int) {
        Self::global().mouse_move(x, y)
    }

    unsafe extern "C" fn set_active_menu(active: c_int) {
        engine().key_clear_states();
        Self::global().set_active_menu(active != 0)
    }

    unsafe extern "C" fn add_server_to_list(adr: netadr_s, info: *const c_char) {
        let info = unsafe { CStr::from_ptr(info).to_str().unwrap() };
        Self::global().add_server_to_list(adr, info)
    }

    unsafe extern "C" fn get_cursor_pos(x: *mut c_int, y: *mut c_int) {
        Self::global().get_cursor_pos(x, y)
    }

    unsafe extern "C" fn set_cursor_pos(x: c_int, y: c_int) {
        Self::global().set_cursor_pos(x, y)
    }

    unsafe extern "C" fn show_cursor(show: c_int) {
        Self::global().show_cursor(show)
    }

    unsafe extern "C" fn char_event(key: c_int) {
        Self::global().char_event(key)
    }

    unsafe extern "C" fn mouse_in_rect() -> c_int {
        Self::global().mouse_in_rect()
    }

    unsafe extern "C" fn is_visible() -> c_int {
        Self::global().is_visible()
    }

    unsafe extern "C" fn credits_active() -> c_int {
        Self::global().credits_active()
    }

    unsafe extern "C" fn final_credits() {
        Self::global().final_credits()
    }
}

#[allow(unused_variables)]
pub trait UiFunctions {
    fn vid_init(&mut self) -> c_int {
        0
    }

    fn shutdown(&mut self) {}

    fn redraw(&mut self, time: f32) {}

    fn key_event(&mut self, key: c_int, down: c_int) {}

    fn mouse_move(&mut self, x: c_int, y: c_int) {}

    fn set_active_menu(&mut self, active: bool) {}

    fn add_server_to_list(&mut self, addr: netadr_s, info: &str) {}

    fn get_cursor_pos(&mut self, x: *mut c_int, y: *mut c_int) {}

    fn set_cursor_pos(&mut self, x: c_int, y: c_int) {}

    fn show_cursor(&mut self, show: c_int) {}

    fn char_event(&mut self, key: c_int) {}

    fn mouse_in_rect(&mut self) -> c_int {
        0
    }

    fn is_visible(&mut self) -> c_int {
        0
    }

    fn credits_active(&mut self) -> c_int {
        0
    }

    fn final_credits(&mut self) {}
}

pub trait MenuApiExtended<T: 'static + UiFunctionsExtended>: MenuApi<T> {
    fn ui_funcs_extened() -> raw::UI_EXTENDED_FUNCTIONS {
        raw::UI_EXTENDED_FUNCTIONS {
            pfnAddTouchButtonToList: Some(Self::add_touch_button_to_list),
            pfnResetPing: Some(Self::reset_ping),
            pfnShowConnectionWarning: Some(Self::show_connection_warning),
            pfnShowUpdateDialog: Some(Self::show_update_dialog),
            pfnShowMessageBox: Some(Self::show_message_box),
            pfnConnectionProgress_Disconnect: Some(Self::connection_progress_disconnect),
            pfnConnectionProgress_Download: Some(Self::connection_progress_download),
            pfnConnectionProgress_DownloadEnd: Some(Self::connection_process_download_end),
            pfnConnectionProgress_Precache: Some(Self::connection_progress_precache),
            pfnConnectionProgress_Connect: Some(Self::connection_progress_connect),
            pfnConnectionProgress_ChangeLevel: Some(Self::connection_progress_change_level),
            pfnConnectionProgress_ParseServerInfo: Some(
                Self::connection_progress_parse_server_info,
            ),
        }
    }

    unsafe extern "C" fn add_touch_button_to_list(
        name: *const c_char,
        texture: *const c_char,
        command: *const c_char,
        color: *mut c_uchar,
        flags: c_int,
    ) {
        Self::global().add_touch_button_to_list(name, texture, command, color, flags)
    }

    unsafe extern "C" fn reset_ping() {
        Self::global().reset_ping()
    }

    unsafe extern "C" fn show_connection_warning() {
        Self::global().show_connection_warning()
    }

    unsafe extern "C" fn show_update_dialog(prefer_store: c_int) {
        Self::global().show_update_dialog(prefer_store)
    }

    unsafe extern "C" fn show_message_box(text: *const c_char) {
        if !text.is_null() {
            let text = CStrThin::from_ptr(text);
            Self::global().show_message_box(text)
        }
    }

    unsafe extern "C" fn connection_progress_disconnect() {
        Self::global().connection_progress_disconnect()
    }

    unsafe extern "C" fn connection_progress_download(
        file_name: *const c_char,
        server_name: *const c_char,
        current: c_int,
        total: c_int,
        comment: *const c_char,
    ) {
        if file_name.is_null() || server_name.is_null() || comment.is_null() {
            warn!("connection_progress_download file_name, server_name or comment is null");
            return;
        }
        let file_name = CStrThin::from_ptr(file_name);
        let server_name = CStrThin::from_ptr(server_name);
        let comment = CStrThin::from_ptr(comment);
        Self::global().connection_progress_download(file_name, server_name, current, total, comment)
    }

    unsafe extern "C" fn connection_process_download_end() {
        Self::global().connection_process_download_end()
    }

    unsafe extern "C" fn connection_progress_precache() {
        Self::global().connection_progress_precache()
    }

    unsafe extern "C" fn connection_progress_connect(server: *const c_char) {
        if !server.is_null() {
            let server = CStrThin::from_ptr(server);
            Self::global().connection_progress_connect(server)
        }
    }

    unsafe extern "C" fn connection_progress_change_level() {
        Self::global().connection_progress_change_level()
    }

    unsafe extern "C" fn connection_progress_parse_server_info(server: *const c_char) {
        if !server.is_null() {
            let server = CStrThin::from_ptr(server);
            Self::global().connection_progress_parse_server_info(server);
        }
    }
}

#[allow(unused_variables)]
pub trait UiFunctionsExtended: UiFunctions {
    fn add_touch_button_to_list(
        &mut self,
        name: *const c_char,
        texture: *const c_char,
        command: *const c_char,
        color: *mut c_uchar,
        flags: c_int,
    ) {
    }
    fn reset_ping(&mut self) {}
    fn show_connection_warning(&mut self) {}
    fn show_update_dialog(&mut self, prefer_store: c_int) {}
    fn show_message_box(&mut self, text: &CStrThin) {}
    fn connection_progress_disconnect(&mut self) {}
    fn connection_progress_download(
        &mut self,
        file_name: &CStrThin,
        server_name: &CStrThin,
        current: c_int,
        total: c_int,
        comment: &CStrThin,
    ) {
    }
    fn connection_process_download_end(&mut self) {}
    fn connection_progress_precache(&mut self) {}
    fn connection_progress_connect(&mut self, server: &CStrThin) {}
    fn connection_progress_change_level(&mut self) {}
    fn connection_progress_parse_server_info(&mut self, server: &CStrThin) {}
}

static UI: SyncOnceCell<RefCell<Ui>> = unsafe { SyncOnceCell::new() };

pub struct Api;

impl MenuApi<Ui> for Api {
    fn global() -> RefMut<'static, Ui> {
        UI.get().unwrap().borrow_mut()
    }
}

impl MenuApiExtended<Ui> for Api {}

#[no_mangle]
pub extern "C" fn GetMenuAPI(
    ui_funcs: Option<&mut xash3d_ui::raw::UI_FUNCTIONS>,
    eng_funcs: Option<&xash3d_ui::raw::ui_enginefuncs_s>,
    globals: Option<&'static xash3d_ui::raw::ui_globalvars_s>,
) -> c_int {
    let (Some(ui_funcs), Some(eng_funcs), Some(globals)) = (ui_funcs, eng_funcs, globals) else {
        return 0;
    };

    *ui_funcs = Api::ui_funcs();
    xash3d_ui::init(eng_funcs, globals);
    crate::logger::init();
    1
}

#[no_mangle]
pub extern "C" fn GetExtAPI(
    version: c_int,
    ui_funcs: Option<&mut xash3d_ui::raw::UI_EXTENDED_FUNCTIONS>,
    eng_funcs: Option<&xash3d_ui::raw::ui_extendedfuncs_s>,
) -> c_int {
    let (Some(ui_funcs), Some(eng_funcs)) = (ui_funcs, eng_funcs) else {
        return 0;
    };

    if version != MENU_EXTENDED_API_VERSION {
        error!(
            "failed to initialize extended menu API. Expected by DLL: {}. Got from engine: {}",
            MENU_EXTENDED_API_VERSION, version
        );
        return 0;
    }
    *ui_funcs = Api::ui_funcs_extened();
    xash3d_ui::init_ext(eng_funcs);
    1
}
