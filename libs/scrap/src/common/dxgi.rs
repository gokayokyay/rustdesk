use crate::dxgi;
use std::io::ErrorKind::{NotFound, TimedOut, WouldBlock};
use std::{io, ops};

pub struct Capturer {
    inner: dxgi::Capturer,
    width: usize,
    height: usize,
}

impl Capturer {
    pub fn new(display: Display, yuv: bool) -> io::Result<Capturer> {
        let width = display.width();
        let height = display.height();
        let inner = dxgi::Capturer::new(display.0, yuv)?;
        Ok(Capturer {
            inner,
            width,
            height,
        })
    }

    pub fn is_gdi(&self) -> bool {
        self.inner.is_gdi()
    }

    pub fn set_gdi(&mut self) -> bool {
        self.inner.set_gdi()
    }

    pub fn cancel_gdi(&mut self) {
        self.inner.cancel_gdi()
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn frame<'a>(&'a mut self, timeout_ms: u32) -> io::Result<Frame<'a>> {
        match self.inner.frame(timeout_ms) {
            Ok(frame) => Ok(Frame(frame)),
            Err(ref error) if error.kind() == TimedOut => Err(WouldBlock.into()),
            Err(error) => Err(error),
        }
    }
}

pub struct Frame<'a>(&'a [u8]);

impl<'a> ops::Deref for Frame<'a> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.0
    }
}

pub struct Display(dxgi::Display);

impl Display {
    pub fn primary() -> io::Result<Display> {
        match dxgi::Displays::new()?.next() {
            Some(inner) => Ok(Display(inner)),
            None => Err(NotFound.into()),
        }
    }

    pub fn all() -> io::Result<Vec<Display>> {
        Ok(dxgi::Displays::new()?.map(Display).collect::<Vec<_>>())
    }

    pub fn width(&self) -> usize {
        self.0.width() as usize
    }

    pub fn height(&self) -> usize {
        self.0.height() as usize
    }

    pub fn name(&self) -> String {
        use std::ffi::OsString;
        use std::os::windows::prelude::*;
        OsString::from_wide(self.0.name())
            .to_string_lossy()
            .to_string()
    }

    pub fn is_online(&self) -> bool {
        self.0.is_online()
    }

    pub fn origin(&self) -> (usize, usize) {
        let o = self.0.origin();
        (o.0 as usize, o.1 as usize)
    }

    // to-do: not found primary display api for dxgi
    pub fn is_primary(&self) -> bool {
        self.name() == Self::primary().unwrap().name()
    }
}
