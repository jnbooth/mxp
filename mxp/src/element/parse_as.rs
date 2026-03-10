use std::str::FromStr;

use crate::parser::{StringVariant, UnrecognizedVariant};

/// The MUD server can tag a line to be parsed in a specific way by the client.
///
/// See [MXP specification: Tag Properties](https://www.zuggsoft.com/zmud/mxp.htm#Tag%20Properties).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ParseAs {
    /// The text for the element is parsed by the automapper as the name of a room.
    RoomName,
    /// he text for the element is parsed by the automapper as the description of a room.
    RoomDesc,
    /// The text for the element is parsed by the automapper as exits for the room.
    RoomExit,
    /// The text for the element is parsed by the automapper as a room number.
    RoomNum,
    /// The text for the element is parsed by as a MUD Prompt.
    Prompt,
}

impl StringVariant for ParseAs {
    type Variant = Self;
    const VARIANTS: &[Self] = &[
        Self::RoomName,
        Self::RoomDesc,
        Self::RoomExit,
        Self::RoomNum,
        Self::Prompt,
    ];
}

impl FromStr for ParseAs {
    type Err = UnrecognizedVariant<Self>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match_ci! {s,
            "roomname" => Ok(Self::RoomName),
            "roomdesc" => Ok(Self::RoomDesc),
            "roomexit" => Ok(Self::RoomExit),
            "roomnum" => Ok(Self::RoomNum),
            "prompt" => Ok(Self::Prompt),
            _ => Err(Self::Err::new(s)),
        }
    }
}
