use super::sound;

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct State<T> {
    focus: T,
}

impl<T> State<T> {
    // pub fn new(focus: T) -> Self {
    //     Self { focus }
    // }

    pub fn focus(&self) -> &T {
        &self.focus
    }

    pub fn set(&mut self, focus: T) {
        self.focus = focus;
    }

    pub fn reset(&mut self)
    where
        T: Default,
    {
        self.set(T::default())
    }

    pub fn confirm(&mut self, focus: T) {
        self.set(focus);
        sound::confirm();
    }

    pub fn confirm_default(&mut self)
    where
        T: Default,
    {
        self.confirm(T::default())
    }

    pub fn deny(&mut self, focus: T) {
        self.set(focus);
        sound::deny();
    }

    pub fn deny_default(&mut self)
    where
        T: Default,
    {
        self.deny(T::default())
    }

    pub fn select(&mut self, focus: T) {
        self.set(focus);
        sound::select_item();
    }

    pub fn cancel(&mut self, focus: T) {
        self.set(focus);
        sound::deny2();
    }

    pub fn cancel_default(&mut self)
    where
        T: Default,
    {
        self.cancel(T::default())
    }

    pub fn prev(&mut self, focus: T) {
        self.set(focus);
        sound::select_prev();
    }

    pub fn next(&mut self, focus: T) {
        self.set(focus);
        sound::select_next();
    }
}

impl<T: Eq> PartialEq<T> for State<T> {
    fn eq(&self, other: &T) -> bool {
        self.focus.eq(other)
    }
}
