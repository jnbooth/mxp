use crate::parse::UnrecognizedVariant;

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

impl_parse_enum!(ParseAs, RoomName, RoomDesc, RoomExit, RoomNum, Prompt);
