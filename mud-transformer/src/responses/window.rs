use std::fmt;

use mxp::escape::ansi::{CSI, OSC, ST};

#[derive(Copy, Clone, Debug)]
pub enum WindowStateReport {
    Open = 1,
    Iconified = 2,
}

impl fmt::Display for WindowStateReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code = *self as u8;
        write!(f, "{CSI}{code}t")
    }
}

#[derive(Copy, Clone, Debug)]
pub struct WindowPositionReport {
    pub x: u16,
    pub y: u16,
}

impl fmt::Display for WindowPositionReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { x, y } = self;
        write!(f, "{CSI}3;{x};{y}t")
    }
}

#[derive(Copy, Clone, Debug)]
pub struct WindowSizeReport {
    pub height: u16,
    pub width: u16,
}

impl fmt::Display for WindowSizeReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { height, width } = self;
        write!(f, "{CSI}4;{height};{width}t")
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TextAreaSizeReport {
    pub height: u16,
    pub width: u16,
}

impl fmt::Display for TextAreaSizeReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { height, width } = self;
        write!(f, "{CSI}8;{height};{width}t")
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ScreenSizeReport {
    pub height: u16,
    pub width: u16,
}

impl fmt::Display for ScreenSizeReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { height, width } = self;
        write!(f, "{CSI}9;{height};{width}t")
    }
}

#[derive(Copy, Clone, Debug)]
pub struct WindowIconLabelReport<'a> {
    label: &'a str,
}

impl fmt::Display for WindowIconLabelReport<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { label } = self;
        write!(f, "{OSC}L{label}{ST}")
    }
}

#[derive(Copy, Clone, Debug)]
pub struct WindowTitleReport<'a> {
    title: &'a str,
}

impl fmt::Display for WindowTitleReport<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { title } = self;
        write!(f, "{OSC}l{title}{ST}")
    }
}
