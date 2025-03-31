use bitflags::bitflags;
use ratatui::layout::Position;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Key {
    Ctrl,
    Alt,
    Shift,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Enter,
    Escape,
    PageUp,
    PageDown,
    Home,
    End,
    Delete,
    Backspace,
    Tab,
    Char(u8),
    Mouse(u8),
    MouseWheelUp(u16),
    MouseWheelDown(u16),

    // pseudo
    MouseWheelLeft(u16),
    MouseWheelRight(u16),

    TouchStart(Position),
    Touch(i32, i32),
    TouchStop(Position),
}

impl Key {
    pub fn is_exec(&self) -> bool {
        matches!(self, Key::Char(b'l') | Key::ArrowRight | Key::Enter)
    }

    pub fn is_back(&self) -> bool {
        matches!(self, Key::Char(b'h' | b'q') | Key::ArrowLeft | Key::Escape)
    }

    pub fn is_prev(&self) -> bool {
        matches!(self, Key::Char(b'k') | Key::ArrowUp)
    }

    pub fn is_next(&self) -> bool {
        matches!(self, Key::Char(b'j') | Key::ArrowDown)
    }
}

impl From<u8> for Key {
    fn from(c: u8) -> Key {
        use xash3d_ui::consts::keys::*;
        match c {
            K_CTRL => Self::Ctrl,
            K_ALT => Self::Alt,
            K_SHIFT => Self::Shift,
            K_RIGHTARROW => Self::ArrowRight,
            K_UPARROW => Self::ArrowUp,
            K_DOWNARROW => Self::ArrowDown,
            K_LEFTARROW => Self::ArrowLeft,
            K_ENTER => Self::Enter,
            K_ESCAPE => Self::Escape,
            K_PGUP => Self::PageUp,
            K_PGDN => Self::PageDown,
            K_HOME => Self::Home,
            K_END => Self::End,
            K_DEL => Self::Delete,
            K_BACKSPACE => Self::Backspace,
            K_TAB => Self::Tab,
            K_MOUSE1..=K_MOUSE5 => Self::Mouse(c - K_MOUSE1),
            K_MWHEELUP => Self::MouseWheelUp(1),
            K_MWHEELDOWN => Self::MouseWheelDown(1),
            _ => Self::Char(c),
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Modifier {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

bitflags! {
    #[derive(Copy, Clone)]
    struct Flags: u8 {
        const DOWN      = 1 << 0;
        const CTRL      = 1 << 1;
        const SHIFT     = 1 << 2;
        const ALT       = 1 << 3;
    }
}

#[derive(Copy, Clone)]
pub struct KeyEvent {
    raw: u8,
    flags: Flags,
    key: Key,
}

impl KeyEvent {
    pub fn with_key(raw: u8, modifier: Modifier, down: bool, key: Key) -> Self {
        let mut flags = Flags::empty();
        flags.set(Flags::DOWN, down);
        flags.set(Flags::CTRL, modifier.ctrl);
        flags.set(Flags::SHIFT, modifier.shift);
        flags.set(Flags::ALT, modifier.alt);

        Self { raw, flags, key }
    }

    pub fn new(raw: u8, modifier: Modifier, down: bool) -> Self {
        Self::with_key(raw, modifier, down, Key::from(raw))
    }

    // TODO: move to separate methods
    pub fn new_touch(modifier: Modifier, key: Key) -> Self {
        Self::with_key(0, modifier, true, key)
    }

    pub fn raw(&self) -> u8 {
        self.raw
    }

    pub fn is_down(&self) -> bool {
        self.flags.intersects(Flags::DOWN)
    }

    pub fn is_up(&self) -> bool {
        !self.is_down()
    }

    pub fn ctrl(&self) -> bool {
        self.flags.intersects(Flags::CTRL)
    }

    pub fn shift(&self) -> bool {
        self.flags.intersects(Flags::SHIFT)
    }

    pub fn alt(&self) -> bool {
        self.flags.intersects(Flags::ALT)
    }

    pub fn key(&self) -> Key {
        self.key
    }
}
