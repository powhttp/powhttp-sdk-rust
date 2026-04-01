
use serde::{Deserialize, Serialize};
use crate::shared::{NamedU8, NamedU16, NamedU32, Side};

/// An HTTP/2 frame observed on one side of the connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Http2Event {
    pub side: Side,
    pub frame: Frame,
}

/// A parsed HTTP/2 frame with its flags and payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    pub flags: Flags,
    pub payload: Payload,
}

/// HTTP/2 frame flags with the raw byte value and the named flags that are set.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Flags {
    pub value: u8,
    pub defined: Vec<NamedU8>,
}

/// The frame-type–specific payload of an HTTP/2 frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", content = "content")]
pub enum Payload {
    Data(Data),
    Headers(Headers),
    Priority(Priority),
    RstStream(RstStream),
    Settings(Settings),
    PushPromise(PushPromise),
    Ping(Ping),
    GoAway(GoAway),
    WindowUpdate(WindowUpdate),
    Continuation(Continuation),
    Unknown(UnknownPayload)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub padding_len: Option<usize>,
    #[serde(with = "hex")]
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Headers {
    pub padding_len: Option<usize>,
    pub priority_info: Option<Priority>,
    #[serde(with = "hex")]
    pub header_block_fragment: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Priority {
    pub is_exclusive: bool,
    pub stream_dependency: u32,
    pub weight: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RstStream {
    pub error_code: NamedU32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub settings: Vec<SettingParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingParameter {
    pub identifier: NamedU16,
    pub value: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushPromise {
    pub padding_len: Option<usize>,
    pub promised_stream_id: u32,
    #[serde(with = "hex")]
    pub header_block_fragment: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ping {
    #[serde(with = "hex")]
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoAway {
    pub last_stream_id: u32,
    pub error_code: NamedU32,
    #[serde(with = "hex")]
    pub debug_data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowUpdate {
    pub window_size_increment: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Continuation {
    #[serde(with = "hex")]
    pub field_block_fragment: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnknownPayload {
    #[serde(with = "hex")]
    pub data: Vec<u8>,
}
