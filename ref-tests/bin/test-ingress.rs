use ic_agent::{Agent, AgentError, Identity, RequestId};
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::ManagementCanister;

fn main() {
    let mut rt = tokio::runtime::Runtime::new().expect("Could not create tokio runtime.");

    rt.block_on(async move {
        println!("It is now: {:?}", std::time::Instant::now());

        let agent1 = create_agent_with_ingress_expiry(
            "http://localhost:8080",
            std::time::Duration::from_secs(20),
        )
        .await
        .unwrap();

        // Create a canister.
        let mgr = ManagementCanister::create(&agent1);
        let _ = mgr
            .provisional_create_canister_with_cycles(None)
            .call_and_wait(delay())
            .await;
    })
}

pub async fn create_agent_with_ingress_expiry(
    url: &str,
    ie: std::time::Duration,
) -> Result<Agent, AgentError> {
    let a = Agent::builder()
        .with_url(url)
        .with_identity(random_ed25519_identity())
        .with_ingress_expiry(Some(ie))
        .build()?;
    a.fetch_root_key().await?;
    Ok(a)
}

// How `Agent` is instructed to wait for update calls.
pub fn delay() -> delay::Delay {
    delay::Delay::builder()
        .throttle(std::time::Duration::from_millis(500))
        .timeout(std::time::Duration::from_secs(60 * 5))
        .build()
}

// Creates an identity to be used with `Agent`.
pub fn random_ed25519_identity() -> impl Identity {
    let rng = ring::rand::SystemRandom::new();
    let key_pair = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng)
        .expect("Could not generate a key pair.");

    ic_agent::identity::BasicIdentity::from_key_pair(
        ring::signature::Ed25519KeyPair::from_pkcs8(key_pair.as_ref())
            .expect("Could not read the key pair."),
    )
}
