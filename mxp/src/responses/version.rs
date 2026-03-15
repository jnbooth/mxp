use std::fmt;

/// Formats a [`<VERSION>`] response.
///
/// [`<VERSION>`]: https://www.zuggsoft.com/zmud/mxp.htm#Version%20Control
///
/// # Examples
///
/// ```
/// use mxp::responses::VersionResponse;
///
/// let response = VersionResponse {
///     client: "myclient",
///     version: "6.07",
///     ..Default::default()
/// };
///
/// assert_eq!(
///     response.to_string(),
///     "\x1B[1z<VERSION MXP=1.0 CLIENT=myclient VERSION=6.07>\r\n"
/// );
///
/// let advanced_response = VersionResponse {
///     client: "myclient",
///     version: "6.07",
///     style: Some("1.05"),
///     registered: Some(true),
/// };
///
/// assert_eq!(
///     advanced_response.to_string(),
///     "\x1B[1z<VERSION MXP=1.0 STYLE=1.05 CLIENT=myclient VERSION=6.07 REGISTERED=yes>\r\n"
/// );
/// ```
#[derive(Copy, Clone, Debug, Default)]
pub struct VersionResponse<'a> {
    /// Name of the MUD client. If more than one word, the value should be in quotes.
    pub client: &'a str,
    /// Version of the MUD client.
    pub version: &'a str,
    /// Current version of the optional style sheet. A MUD sets a style-sheet version number by
    /// sending the `<VERSION styleversion>` tag to the client. The client caches this version
    /// information and returns it when requested by a plain `<VERSION>` request.
    pub style: Option<&'a str>,
    /// Used to detect if the player is using a registered version of the client.
    pub registered: Option<bool>,
}

impl fmt::Display for VersionResponse<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            client: name,
            version,
            style,
            registered,
        } = self;
        let registered = match registered {
            Some(true) => " REGISTERED=yes",
            Some(false) => " REGISTERED=no",
            None => "",
        };
        write!(f, "\x1B[1z<VERSION MXP=1.0 ")?;
        if let Some(style) = style {
            write!(f, "STYLE={style} ")?;
        }
        write!(f, "CLIENT={name} VERSION={version}{registered}>\r\n")
    }
}
