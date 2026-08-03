use codex_http_client::ClientRouteClass;
use codex_http_client::HttpClientFactory;
use codex_http_client::RouteAwareClientPool;
use codex_protocol::account::PlanType;
use rand::Rng;
use serde::Deserialize;
use std::sync::OnceLock;
use std::time::Duration;

const USELESS_FACT_URL: &str = "https://uselessfacts.jsph.pl/api/v2/facts/random";
const FACT_FETCH_TIMEOUT: Duration = Duration::from_millis(/*millis*/ 800);

const CAT_FACTS: &[&str] = &[
    "Cats have 3 eyelids.",
    "Cats walk on their toes.",
    "Most cats adore sardines.",
    "Cats dislike citrus scent.",
    "Cats have supersonic hearing.",
    "A group of cats is called a clowder.",
    "A cat's urine glows under a black light.",
    "Cats can be right-pawed or left-pawed.",
    "A tiger's stripes are like fingerprints.",
    "A domestic cat can run at speeds of 30 mph.",
    "Cats take between 20 and 40 breaths per minute.",
    "The cat's tail is used to maintain balance.",
    "A cat can jump 5 times as high as it is tall.",
    "Cats only sweat through their paws.",
    "The technical term for a cat's hairball is a bezoar.",
    "Cats control their outer ears using 32 muscles; humans use 6.",
    "A cat can make more than 100 different sounds.",
    "Cats have 300 million neurons; dogs have about 160 million.",
    "A cat has about 12 whiskers on each side of its face.",
    "A cat's nose pattern is as unique as a human fingerprint.",
    "Cats lap liquid from the underside of their tongue.",
    "A cat has more bones than a human: 230 compared with 206.",
    "Cheetahs do not roar; instead, they purr.",
    "A cat's heart beats nearly twice as fast as a human heart.",
];

static STARTUP_FACT: OnceLock<String> = OnceLock::new();

#[derive(Deserialize)]
struct UselessFactResponse {
    text: String,
}

/// Fetch the startup fact once, falling back to a bundled cat fact without network access.
pub(crate) async fn prewarm(http_client_factory: HttpClientFactory) {
    if STARTUP_FACT.get().is_some() {
        return;
    }

    let fact = tokio::time::timeout(
        FACT_FETCH_TIMEOUT,
        fetch_remote_fact(http_client_factory),
    )
    .await
    .ok()
    .flatten()
    .unwrap_or_else(random_cat_fact);
    let _ = STARTUP_FACT.set(fact);
}

/// Return the fact selected during startup. Plan and Fast mode do not affect local tips.
pub(crate) fn get_tooltip(_plan: Option<PlanType>, _fast_mode_enabled: bool) -> Option<String> {
    Some(
        STARTUP_FACT
            .get()
            .cloned()
            .unwrap_or_else(random_cat_fact),
    )
}

async fn fetch_remote_fact(http_client_factory: HttpClientFactory) -> Option<String> {
    let client = RouteAwareClientPool::new(http_client_factory, ClientRouteClass::Other);
    let response = client.get(USELESS_FACT_URL).send().await.ok()?;
    let fact = response
        .error_for_status()
        .ok()?
        .json::<UselessFactResponse>()
        .await
        .ok()?;
    let text = fact.text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

fn random_cat_fact() -> String {
    let mut rng = rand::rng();
    CAT_FACTS[rng.random_range(0..CAT_FACTS.len())].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn bundled_cat_fact_is_not_empty() {
        assert!(!random_cat_fact().is_empty());
    }

    #[test]
    fn deserializes_the_remote_fact_shape() {
        let response = serde_json::from_str::<UselessFactResponse>(
            r#"{"id":"example","text":"Cats have 3 eyelids.","language":"en"}"#,
        )
        .expect("sample response should deserialize");

        assert_eq!(response.text, "Cats have 3 eyelids.");
    }
}
