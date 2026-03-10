macro_rules! impl_into_owned {
    ($t:ident) => {
        impl $t<&str> {
            pub fn into_owned(self) -> $t<String> {
                self.map_text(ToOwned::to_owned)
            }
        }

        impl $t<std::borrow::Cow<'_, str>> {
            pub fn into_owned(self) -> $t<String> {
                self.map_text(Cow::into_owned)
            }
        }
    };
}

mod color;
pub use color::Color;

mod dest;
pub use dest::Dest;

mod expire;
pub use expire::Expire;

mod filter;
pub use filter::Filter;

mod font;
pub use font::{FgColor, Font, FontEffect, FontStyle};

mod heading;
pub use heading::Heading;

mod frame;
pub use frame::{Frame, FrameAction, FrameLayout};

mod gauge;
pub use gauge::Gauge;

mod hyperlink;
pub(crate) use hyperlink::Hyperlink;

mod image;
pub use image::Image;

mod link;
pub use link::{Link, LinkPrompt, SendTo};

mod mxp;
pub use mxp::Mxp;

mod relocate;
pub use relocate::Relocate;

mod send;
pub(crate) use send::Send;

mod sound;
pub use sound::{AudioContinuation, AudioRepetition, Music, Sound};

mod stat;
pub use stat::Stat;

mod style_version;
pub use style_version::StyleVersion;

mod support;
pub use support::Support;

mod var;
pub use var::Var;
