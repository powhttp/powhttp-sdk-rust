use serde::{Deserialize, Serialize};
use super::shared::{NamedU8, NamedU16, Side};
use crate::serde_helpers::hex_2d;

/// A TLS record-layer event observed on one side of the connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TlsEvent {
    pub side: Side,
    pub message: TlsMessage,
}

/// Top-level TLS message type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "content")]
pub enum TlsMessage {
    Alert(Alert),
    ChangeCipherSpec(ChangeCipherSpec),
    Handshake(HandshakeMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub level: NamedU8,
    pub description: NamedU8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeCipherSpec {
    #[serde(with = "hex")]
    pub data: Vec<u8>,
}

/// A parsed TLS handshake message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "content")]
pub enum HandshakeMessage {
    ClientHello(ClientHello),
    ServerHello(ServerHello),
    NewSessionTicket(NewSessionTicket),
    EncryptedExtensions(EncryptedExtensions),
    Certificate(Certificate),
    ServerKeyExchange(ServerKeyExchange),
    CertificateRequest(CertificateRequest),
    ServerHelloDone,
    CertificateVerify(CertificateVerify),
    ClientKeyExchange(ClientKeyExchange),
    Finished(Finished),
    CompressedCertificate(CompressedCertificate),
}

/// TLS ClientHello message fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientHello {
    pub version: NamedU16,
    #[serde(with = "hex")]
    pub random: [u8; 32],
    #[serde(with = "hex")]
    pub session_id: Vec<u8>,
    pub cipher_suites: Vec<NamedU16>,
    pub compression_methods: Vec<NamedU8>,
    pub extensions: Vec<ClientExtension>,
}

/// TLS ServerHello message fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerHello {
    pub version: NamedU16,
    #[serde(with = "hex")]
    pub random: [u8; 32],
    #[serde(with = "hex")]
    pub session_id: Vec<u8>,
    pub cipher_suite: NamedU16,
    pub compression_method: NamedU8,
    pub extensions: Vec<ServerExtension>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSessionTicket {
    #[serde(with = "hex")]
    pub ticket: Vec<u8>,
}

/// A TLS extension from a ClientHello message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientExtension {
    pub extension_type: NamedU16,
    pub extension_data: ClientExtensionData,
}

/// Parsed data for a client TLS extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "content")]
pub enum ClientExtensionData {
    Grease(Grease),
    SignedCertificateTimestamp,
    StatusRequest(CertificateStatusRequest),
    Alpn(Alpn),
    Ech(Ech),
    CompressCertificate(CompressCertificateExt),
    SessionTicket(SessionTicket),
    SupportedGroups(SupportedGroups),
    RenegotiationInfo(RenegotiationInfo),
    ExtendedMasterSecret,
    KeyShare(KeyShareClientHello),
    SignatureAlgorithms(SignatureAlgorithms),
    ServerName(ServerNameList),
    EcPointFormats(EcPointFormats),
    SupportedVersions(ClientSupportedVersions),
    PskKeyExchangeModes(PskKeyExchangeModes),
    ApplicationSettingsSupport(ApplicationSettingsSupport),
    ApplicationSettings(ApplicationSettings),
    EncryptThenMac,
    CertificateAuthorities(CertificateAuthorities),
    Padding(Padding),
    Unknown(UnknownExtension),
}

/// A TLS extension from a ServerHello message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerExtension {
    pub extension_type: NamedU16,
    pub extension_data: ServerExtensionData,
}

/// Parsed data for a server TLS extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "content")]
pub enum ServerExtensionData {
    ServerName,
    SignatureAlgorithms(SignatureAlgorithms),
    KeyShareServerHello(KeyShareServerHello),
    KeyShareHelloRetryRequest(KeyShareHelloRetryRequest),
    SupportedVersions(ServerSupportedVersions),
    ExtendedMasterSecret,
    Alpn(Alpn),
    Ech(Ech),
    ApplicationSettings(ApplicationSettings),
    SessionTicket(SessionTicket),
    CertificateAuthorities(CertificateAuthorities),
    Unknown(UnknownExtension),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Grease {
    #[serde(with = "hex")]
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateStatusRequest {
    pub status_type: NamedU8,
    pub request: StatusRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "content")]
pub enum StatusRequest {
    Ocsp(OcspStatusRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcspStatusRequest {
    #[serde(with = "hex_2d")]
    pub responder_id_list: Vec<Vec<u8>>,
    #[serde(with = "hex")]
    pub extensions: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alpn {
    pub protocols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ech {
    #[serde(with = "hex")]
    pub ech: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressCertificateExt {
    pub algorithms: Vec<NamedU16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTicket {
    #[serde(with = "hex")]
    pub ticket: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportedGroups {
    pub named_group_list: Vec<NamedU16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenegotiationInfo {
    #[serde(with = "hex")]
    pub renegotiated_connection: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyShareClientHello {
    pub client_shares: Vec<KeyShareEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyShareEntry {
    pub group: NamedU16,
    #[serde(with = "hex")]
    pub key_exchange: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureAlgorithms {
    pub supported_signature_algorithms: Vec<NamedU16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerNameList {
    pub server_name_list: Vec<ServerName>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerName {
    pub name_type: NamedU8,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EcPointFormats {
    pub ec_point_format_list: Vec<NamedU8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientSupportedVersions {
    pub versions: Vec<NamedU16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerSupportedVersions {
    pub selected_version: NamedU16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PskKeyExchangeModes {
    pub ke_modes: Vec<NamedU8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationSettingsSupport {
    pub supported_protocols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationSettings {
    #[serde(with = "hex")]
    pub settings: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateAuthorities {
    #[serde(with = "hex_2d")]
    pub authorities: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Padding {
    #[serde(with = "hex")]
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnknownExtension {
    #[serde(with = "hex")]
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyShareServerHello {
    pub server_share: KeyShareEntry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyShareHelloRetryRequest {
    pub selected_group: NamedU16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "content")]
pub enum EncryptedExtensions {
    Client(Vec<ClientExtension>),
    Server(Vec<ServerExtension>),
}

/// TLS Certificate message in either TLS 1.2 or TLS 1.3 format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "content")]
pub enum Certificate {
    Tls13(Tls13Certificate),
    Tls12(Tls12Certificate),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tls13Certificate {
    #[serde(with = "hex")]
    pub certificate_request_context: Vec<u8>,
    pub certificate_list: Vec<CertificateEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateEntry {
    #[serde(with = "hex")]
    pub cert_data: Vec<u8>,
    #[serde(with = "hex")]
    pub extensions: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tls12Certificate {
    #[serde(with = "hex_2d")]
    pub certificates: Vec<Vec<u8>>,
}

/// TLS ServerKeyExchange for either DH or ECDH key exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", content = "content")]
pub enum ServerKeyExchange {
    Dh(DhServerKeyExchange),
    Ecdh(EcdhServerKeyExchange),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DhServerKeyExchange {
    pub params: ServerDhParams,
    pub signed_params: Option<DigitallySigned>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerDhParams {
    #[serde(with = "hex")]
    pub p: Vec<u8>,
    #[serde(with = "hex")]
    pub g: Vec<u8>,
    #[serde(with = "hex")]
    pub ys: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DigitallySigned {
    pub algorithm: NamedU16,
    #[serde(with = "hex")]
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EcdhServerKeyExchange {
    pub params: ServerEcdhParams,
    pub signed_params: Option<DigitallySigned>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerEcdhParams {
    pub curve_params: EcParams,
    #[serde(with = "hex")]
    pub public: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", content = "content")]
pub enum EcParams {
    ExplicitPrime(ExplicitPrimeParams),
    #[serde(rename = "explicit_char2")]
    ExplicitChar2(ExplicitChar2Params),
    NamedCurve(NamedCurveParams),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplicitPrimeParams {
    #[serde(with = "hex")]
    pub p: Vec<u8>,
    pub curve: EcCurve,
    #[serde(with = "hex")]
    pub base: Vec<u8>,
    #[serde(with = "hex")]
    pub order: Vec<u8>,
    #[serde(with = "hex")]
    pub cofactor: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplicitChar2Params {
    pub m: u16,
    pub basis: Char2Basis,
    pub curve: EcCurve,
    #[serde(with = "hex")]
    pub base: Vec<u8>,
    #[serde(with = "hex")]
    pub order: Vec<u8>,
    #[serde(with = "hex")]
    pub cofactor: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamedCurveParams {
    pub named_curve: NamedU16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EcCurve {
    #[serde(with = "hex")]
    pub a: Vec<u8>,
    #[serde(with = "hex")]
    pub b: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", content = "content")]
pub enum Char2Basis {
    Trinomial(TrinomialBasis),
    Pentanomial(PentanomialBasis),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrinomialBasis {
    #[serde(with = "hex")]
    pub k: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PentanomialBasis {
    #[serde(with = "hex")]
    pub k1: Vec<u8>,
    #[serde(with = "hex")]
    pub k2: Vec<u8>,
    #[serde(with = "hex")]
    pub k3: Vec<u8>,
}

/// TLS ClientKeyExchange for DH, ECDH or RSA key exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", content = "content")]
pub enum ClientKeyExchange {
    Dh(DhClientKeyExchange),
    Ecdh(EcdhClientKeyExchange),
    Rsa(RsaClientKeyExchange),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DhClientKeyExchange {
    #[serde(with = "hex")]
    pub yc: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EcdhClientKeyExchange {
    #[serde(with = "hex")]
    pub yc: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RsaClientKeyExchange {
    #[serde(with = "hex")]
    pub encrypted_pre_master_secret: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", content = "content")]
pub enum CertificateRequest {
    Tls13(Tls13CertificateRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tls13CertificateRequest {
    #[serde(with = "hex")]
    pub certificate_request_context: Vec<u8>,
    pub extensions: Vec<ServerExtension>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateVerify {
    pub algorithm: NamedU16,
    #[serde(with = "hex")]
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finished {
    #[serde(with = "hex")]
    pub verify_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressedCertificate {
    pub algorithm: NamedU16,
    pub uncompressed_len: usize,
    #[serde(with = "hex")]
    pub compressed_certificate_message: Vec<u8>,
}
