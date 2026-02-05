use std::iter::FusedIterator;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RefreshRate {
    At50Hz = 1,
    At60Hz,
    #[default]
    AtLeast70Hz,
}

impl RefreshRate {
    pub(crate) const fn from_code(code: Option<u16>) -> Option<Self> {
        match code {
            None | Some(0 | 3) => Some(Self::AtLeast70Hz),
            Some(1) => Some(Self::At50Hz),
            Some(2) => Some(Self::At60Hz),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WindowOp {
    /// Iconify or de-iconify the window.
    SetIconify(bool),
    /// Move window to [x, y].
    SetPosition { x: u16, y: u16 },
    /// Resize the xterm window to height and width in pixels.
    SetSize { height: u16, width: u16 },
    /// Raise the xterm window to the front of the stacking order.
    Raise,
    /// Lower the xterm window to the bottom of the stacking order.
    Lower,
    /// Refresh the xterm window.
    Refresh,
    /// Resize the text area to [height;width] in characters.
    SetTextAreaSize { height: u16, width: u16 },
    /// Restore maximized window.
    Restore,
    /// Maximize window (i.e., resize to screen size).
    Maximize,
    /// Report xterm window state with [`WindowStateReport`](crate::responses::WindowStateReport).
    ReportState,
    /// Report xterm window position with [`WindowPositionReport`](crate::responses::WindowPositionReport).
    ReportPosition,
    /// Report xterm window in pixels with [`WindowSizeReport`](crate::responses::WindowSizeReport).
    ReportSize,
    /// Report the size of the text area in characters with [`TextAreaSizeReport`](crate::responses::TextAreaSizeReport).
    ReportTextAreaSize,
    /// Report the size of the screen in characters with [`ScreenSizeReport`](crate::responses::ScreenSizeReport).
    ReportScreenSize,
    /// Report xterm window's icon label with [`WindowIconLabelReport`](crate::responses::WindowIconLabelReport).
    ReportIconLabel,
    /// Report xterm window's title with [`WindowTitleReport`](crate::responses::WindowTitleReport).
    ReportTitle,
    /// DECSLPP (Set Lines Per Page)
    SetLines(u16),
}

impl WindowOp {
    pub(crate) fn parse<I>(iter: I) -> WindowOpIter<I::IntoIter>
    where
        I: IntoIterator<Item = Option<u16>>,
    {
        WindowOpIter {
            inner: iter.into_iter(),
        }
    }
}

pub(crate) struct WindowOpIter<I> {
    inner: I,
}

impl<I> Iterator for WindowOpIter<I>
where
    I: Iterator<Item = Option<u16>>,
{
    type Item = WindowOp;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(Some(code)) = self.inner.next() {
            return Some(match code {
                1 => WindowOp::SetIconify(false),
                2 => WindowOp::SetIconify(true),
                3 => WindowOp::SetPosition {
                    x: self.inner.next()??,
                    y: self.inner.next()??,
                },
                4 => WindowOp::SetSize {
                    height: self.inner.next()??,
                    width: self.inner.next()??,
                },
                5 => WindowOp::Raise,
                6 => WindowOp::Lower,
                7 => WindowOp::Refresh,
                8 => WindowOp::SetTextAreaSize {
                    height: self.inner.next()??,
                    width: self.inner.next()??,
                },
                9 => WindowOp::Restore,
                10 => WindowOp::Maximize,
                11 => WindowOp::ReportState,
                13 => WindowOp::ReportPosition,
                14 => WindowOp::ReportSize,
                18 => WindowOp::ReportTextAreaSize,
                19 => WindowOp::ReportScreenSize,
                20 => WindowOp::ReportIconLabel,
                21 => WindowOp::ReportTitle,
                24.. => WindowOp::SetLines(self.inner.next()??),
                _ => continue,
            });
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.inner.size_hint();
        (0, upper)
    }
}

impl<I> FusedIterator for WindowOpIter<I> where I: Iterator<Item = Option<u16>> {}
