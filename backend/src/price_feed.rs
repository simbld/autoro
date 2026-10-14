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

/// Dernier prix connu d'un instrument, reconstitué au fil des deltas.
#[derive(Debug, Default, Clone)]
pub struct Quote {
	pub ask: Option<f64>,
	pub bid: Option<f64>,
}

impl Quote {
	/// Applique un delta : ne met à jour QUE les champs présents.
	pub fn apply(&mut self, delta: &RateDelta) {
		if let Some(ask) = &delta.ask {
			if let Some(price) = ask.parse::<f64>().ok() {
				self.ask = Some(price);
			}
		}

		if let Some(bid) = &delta.bid {
			if let Some(price) = bid.parse::<f64>().ok() {
				self.bid = Some(price);
			}
		}
	}
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
		// Example: A WebSocket message containing only the Ask price.
		let brut = r#"{"messages":[{"topic":"instrument:100063","content":"{\"Ask\":\"120.7908\",\"Date\":\"2026-10-04T04:31:18Z\",\"UnitMarginAsk\":\"120.7908\",\"PriceRateID\":\"159138830098\"}"}]}"#;

		let push: WsPush = serde_json::from_str(brut).unwrap();
		let delta: RateDelta = serde_json::from_str(&push.messages[0].content).unwrap();

		assert_eq!(delta.ask, Some(String::from("120.7908")));
		assert_eq!(delta.bid, None);
	}
}