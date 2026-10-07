use serde::Deserialize;

/// Enveloppe d'un push eToro : `{ "messages": [...] }`
#[derive(Debug, Deserialize)]
pub struct WsPush {
    pub messages: Vec<WsMessage>,
}

#[derive(Debug, Deserialize)]
pub struct WsMessage {
    pub topic: String,
    /// JSON encodé en string → nécessite un 2ᵉ `serde_json::from_str`
    pub content: String,
}

/// Contenu d'un `Trading.Instrument.Rate` : snapshot complet OU delta partiel.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RateDelta {
    pub ask: Option<String>,
    pub bid: Option<String>,
}
