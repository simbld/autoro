#![allow(dead_code)]

use serde::Deserialize;
use std::collections::HashMap;

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
        if let Some(ask) = &delta.ask
            && let Some(price) = ask.parse::<f64>().ok()
        {
            self.ask = Some(price);
        }

        if let Some(bid) = &delta.bid
            && let Some(price) = bid.parse::<f64>().ok()
        {
            self.bid = Some(price);
        }
    }
}

pub fn instrument_id(topic: &str) -> Option<i64> {
    let reste = topic.strip_prefix("instrument:")?;
    reste.parse::<i64>().ok()
}

/// Traite un message texte reçu d'eToro et met à jour les prix connus.
pub fn handle_text(text: &str, quotes: &mut HashMap<i64, Quote>) {
    let Ok(push) = serde_json::from_str::<WsPush>(text) else {
        return;
    };

    for message in push.messages {
        let Some(id) = instrument_id(&message.topic) else {
            continue;
        };

        let Ok(delta) = serde_json::from_str::<RateDelta>(&message.content) else {
            continue;
        };
        quotes.entry(id).or_default().apply(&delta);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_sans_prix_donne_none() {
        // Message réel copié depuis ta console
        let brut = r#"{"messages":[{"topic":"instrument:100063","content":"{\"Date\":\"2026-10-04T04:31:21Z\",\"PriceRateID\":\"159138830912\"}"}]}"#;

        let push: WsPush = serde_json::from_str(brut).unwrap(); // 1er parse
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

    #[test]
    fn delta_ask_seul_conserve_bid() {
        let mut quote = Quote::default();
        let snapshot = RateDelta {
            ask: Some("120.79".to_string()),
            bid: Some("120.77".to_string()),
        };

        quote.apply(&snapshot);

        let delta = RateDelta {
            ask: Some("121".to_string()),
            bid: None,
        };

        quote.apply(&delta);

        assert_eq!(quote.ask, Some(121.00));
        assert_eq!(quote.bid, Some(120.77));
    }

    #[test]
    fn extrait_id_depuis_topic() {
        assert_eq!(instrument_id("instrument:100063"), Some(100063));
        assert_eq!(instrument_id("candle:100063"), None);
        assert_eq!(instrument_id("instrument:abc"), None);
    }

	#[test]
	fn sequence_complete_etoro() {
		let mut quotes = HashMap::new();

		// 1. Réponse d'auth : pas de "messages" → doit être ignorée sans planter
		handle_text(r#"{"id":"x","success":true,"operation":"Authenticate"}"#, &mut quotes);

		// 2. Snapshot SOL : Ask + Bid
		handle_text(r#"{"messages":[{"topic":"instrument:100063","content":"{\"Ask\":\"120.79\",\"Bid\":\"120.77\"}"}]}"#, &mut quotes);

		// 3. Delta SOL : seulement Ask
		handle_text(r#"{"messages":[{"topic":"instrument:100063","content":"{\"Ask\":\"121\"}"}]}"#, &mut quotes);

		assert_eq!(quotes.len(), 1);
		assert_eq!(quotes[&100063].ask, Some(121.00));
		assert_eq!(quotes[&100063].bid, Some(120.77));
	}
}
