use ruma::api::exports::http::uri;
use ruma::identifiers::{EventId, RoomId, UserId};

/// An item that can be represented using a matrix.to URL.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum MatrixToItem<'a> {
    /// An event, since event IDs are room local a RoomId is required.
    Event(&'a RoomId, &'a EventId),
    /// An ID of an user.
    User(&'a UserId),
    /// A ID to a group, the first character must be an +.
    Group(&'a String),
}

impl<'a> MatrixToItem<'a> {
    /// Convert the current `MatrixToItem` into a `uri::Uri`.
    pub fn as_url(&self) -> uri::Uri {
        let slug = match self {
            MatrixToItem::Event(room_id, event_id) => format!("{}/{}", room_id, event_id),
            MatrixToItem::User(user_id) => user_id.to_string(),
            MatrixToItem::Group(group_id) => group_id.to_string(),
        };

        let s = format!("https://matrix.to/#/{}", slug);
        s.parse().unwrap()
    }
}

/// An error from converting an MXC URI to a HTTP URL.
#[derive(Debug)]
pub enum MxcConversionError {
    /// The given MXC URI isn't actually an MXC URI.
    NonMxc,
    /// The given MXC URI is malformed.
    InvalidMxc,
    /// There was an error parsing the resulting URL into an URI object.
    UriParseError(uri::InvalidUri),
}

impl From<uri::InvalidUri> for MxcConversionError {
    fn from(err: uri::InvalidUri) -> Self {
        MxcConversionError::UriParseError(err)
    }
}

/// Convert the given MXC URI into a HTTP URL, using the given `homeserver_url` as the host to the
/// MXC content.
pub fn mxc_to_url(
    homeserver_url: &uri::Uri,
    mxc_uri: &uri::Uri,
) -> Result<uri::Uri, MxcConversionError> {
    if mxc_uri.scheme_str().unwrap() != "mxc" {
        return Err(MxcConversionError::NonMxc);
    }

    let server_name = mxc_uri.host().ok_or(MxcConversionError::InvalidMxc)?;
    let id = &mxc_uri.path()[1..];

    let res = format!(
        "{}_matrix/media/r0/download/{}/{}",
        homeserver_url, server_name, id
    );
    Ok(res.parse()?)
}
