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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn delta_sans_prix_donne_none() {
		// Message réel copié depuis ta console
		let brut = r#"{"messages":[{"topic":"instrument:100063","content":"{\"Date\":\"2026-10-04T04:31:21Z\",\"PriceRateID\":\"159138830912\"}"}]}"#;

		let push: WsPush = serde_json::from_str(brut).unwrap();                    // 1er parse
		let delta: RateDelta = serde_json::from_str(&push.messages[0].content).unwrap(); // 2ᵉ parse

		assert_eq!(delta.ask, None);
		assert_eq!(delta.bid, None);
	}

	#[test]
	fn delta_avec_ask_seul() {
		// Message réel copié depuis la console
		let brut = r#"{"messages":[{"Ask":"120.7908","content":"{\"Date\":\"2026-10-04T04:31:18Z\",\"UnitMarginAsk\":\"120.7908\",\"PriceRateID\":\"159138830098\"}"}]}"#;

		let push: WsPush = serde_json::from_str(brut).unwrap();
		let delta: RateDelta = serde_json::from_str(&push.messages[0].content).unwrap();

		assert_eq!(delta.ask, Some(String::from("120.7908")));
		assert_eq!(delta.bid, None);
	}
}