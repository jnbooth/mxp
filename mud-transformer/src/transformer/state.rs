use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StateLock {
    inner: Option<mxp::State>,
}

impl StateLock {
    #[inline]
    #[track_caller]
    pub fn take(&mut self) -> mxp::State {
        self.inner
            .take()
            .expect("attempted to borrow MXP state during another borrow")
    }

    pub fn set(&mut self, state: mxp::State) {
        self.inner = Some(state);
    }
}

impl From<mxp::State> for StateLock {
    fn from(value: mxp::State) -> Self {
        Self { inner: Some(value) }
    }
}

impl Deref for StateLock {
    type Target = mxp::State;

    #[inline]
    #[track_caller]
    fn deref(&self) -> &Self::Target {
        self.inner
            .as_ref()
            .expect("attempted to access MXP state during borrow")
    }
}

impl DerefMut for StateLock {
    #[inline]
    #[track_caller]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
            .as_mut()
            .expect("attempted to mutate MXP state during borrow")
    }
}
