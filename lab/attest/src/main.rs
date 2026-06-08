//! ENTERPRISE.2 — DSSE attestation signer/verifier (ed25519, Rust-native).
//!
//! Doctrine: optional cryptographic verification of generated attestation artifacts. UNSIGNED is a valid,
//! EXPLICIT honest state (`unsigned_no_key_configured`) — never a failure for open-source replay. SIGNED
//! means the configured key verifies the DSSE PAE over the EXACT attestation payload. No fake green: every
//! failure mode is a distinct status, not a silent pass. Verification is byte-exact and reproducible.

use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_compact::{KeyPair, PublicKey, Seed, Signature};
use serde::Deserialize;

// The ONLY admitted verification states.
const UNSIGNED: &str = "unsigned_no_key_configured";
const VERIFIED: &str = "signed_verified";
const UNVERIFIED: &str = "signed_unverified";
const KEY_MISMATCH: &str = "signed_key_mismatch";
const PAYLOAD_MISMATCH: &str = "signed_payload_mismatch";
// `verification_tool_unavailable` is the CALLER's state when this binary is absent — not produced here.

#[derive(Deserialize)]
struct Sig {
    #[serde(default)]
    keyid: String,
    #[serde(default)]
    sig: String,
}
#[derive(Deserialize)]
struct Envelope {
    #[serde(rename = "payloadType")]
    payload_type: String,
    payload: String,
    #[serde(default)]
    signatures: Vec<Sig>,
}

/// DSSE Pre-Authentication Encoding — the exact bytes signed.
fn pae(payload_type: &str, payload: &[u8]) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(b"DSSEv1 ");
    v.extend_from_slice(payload_type.len().to_string().as_bytes());
    v.push(b' ');
    v.extend_from_slice(payload_type.as_bytes());
    v.push(b' ');
    v.extend_from_slice(payload.len().to_string().as_bytes());
    v.push(b' ');
    v.extend_from_slice(payload);
    v
}

fn keypair_from_seed_hex(seed_hex: &str) -> KeyPair {
    let seed_bytes = hex::decode(seed_hex.trim()).expect("seed hex");
    let seed = Seed::from_slice(&seed_bytes).expect("32-byte seed");
    KeyPair::from_seed(seed)
}

/// Build a signed DSSE envelope JSON (deterministic ed25519: Noise = None).
fn sign(payload: &[u8], payload_type: &str, kp: &KeyPair) -> String {
    let sig = kp.sk.sign(pae(payload_type, payload), None);
    let keyid = hex::encode(kp.pk.as_ref());
    format!(
        "{{\"payloadType\":\"{}\",\"payload\":\"{}\",\"signatures\":[{{\"keyid\":\"{}\",\"sig\":\"{}\"}}]}}",
        payload_type,
        STANDARD.encode(payload),
        keyid,
        STANDARD.encode(sig.as_ref()),
    )
}

/// Return one of the verification states (mirrors the gate's allowed outcomes).
fn verify(env: &Envelope, expected_payload: &[u8], pk: Option<&PublicKey>, expected_keyid: Option<&str>) -> &'static str {
    let payload = match STANDARD.decode(env.payload.as_bytes()) {
        Ok(p) => p,
        Err(_) => return PAYLOAD_MISMATCH,
    };
    // The envelope payload MUST be the exact attestation we expect (binds DSSE to the in-toto/casefile).
    if payload != expected_payload {
        return PAYLOAD_MISMATCH;
    }
    let sig_b64 = env.signatures.first().map(|s| s.sig.as_str()).unwrap_or("");
    if sig_b64.is_empty() {
        return if pk.is_none() { UNSIGNED } else { UNVERIFIED };
    }
    let pk = match pk {
        Some(p) => p,
        None => return UNVERIFIED, // a signature is present but no key is configured to check it
    };
    if let Some(want) = expected_keyid {
        let have = env.signatures.first().map(|s| s.keyid.as_str()).unwrap_or("");
        if have != want {
            return KEY_MISMATCH;
        }
    }
    let sig_bytes = match STANDARD.decode(sig_b64.as_bytes()) {
        Ok(b) => b,
        Err(_) => return UNVERIFIED,
    };
    let sig = match Signature::from_slice(&sig_bytes) {
        Ok(s) => s,
        Err(_) => return UNVERIFIED,
    };
    match pk.verify(pae(&env.payload_type, &payload), &sig) {
        Ok(()) => VERIFIED,
        Err(_) => UNVERIFIED,
    }
}

fn pk_from_hex(h: &str) -> PublicKey {
    PublicKey::from_slice(&hex::decode(h.trim()).expect("pk hex")).expect("pk")
}

fn selftest() -> i32 {
    // deterministic test keys from fixed seeds (no RNG; reproducible)
    let test = keypair_from_seed_hex(&"11".repeat(32));
    let wrong = keypair_from_seed_hex(&"22".repeat(32));
    let test_kid = hex::encode(test.pk.as_ref());
    let wrong_kid = hex::encode(wrong.pk.as_ref());
    let payload = br#"{"_type":"https://in-toto.io/Statement/v1","subject":[]}"#;
    let pt = "application/vnd.in-toto+json";

    let env_json = sign(payload, pt, &test);
    let env: Envelope = serde_json::from_str(&env_json).unwrap();

    let mut fails = 0;
    let mut check = |name: &str, got: &str, want: &str| {
        if got != want {
            eprintln!("  FAIL {name}: got {got}, want {want}");
            fails += 1;
        } else {
            eprintln!("  ok   {name}: {got}");
        }
    };
    // 1. honest signed path
    check("signed_verified", verify(&env, payload, Some(&test.pk), Some(&test_kid)), VERIFIED);
    // 2. wrong expected payload
    check("signed_payload_mismatch", verify(&env, b"{}", Some(&test.pk), Some(&test_kid)), PAYLOAD_MISMATCH);
    // 3. tampered signature (flip one byte) -> still decodes, fails verify
    let tampered = {
        let mut e: serde_json::Value = serde_json::from_str(&env_json).unwrap();
        let s = e["signatures"][0]["sig"].as_str().unwrap().to_string();
        let mut raw = STANDARD.decode(s.as_bytes()).unwrap();
        raw[0] ^= 0x01;
        e["signatures"][0]["sig"] = serde_json::Value::String(STANDARD.encode(&raw));
        e.to_string()
    };
    let env_t: Envelope = serde_json::from_str(&tampered).unwrap();
    check("signed_unverified (tampered sig)", verify(&env_t, payload, Some(&test.pk), Some(&test_kid)), UNVERIFIED);
    // 4. wrong configured keyid
    check("signed_key_mismatch", verify(&env, payload, Some(&test.pk), Some(&wrong_kid)), KEY_MISMATCH);
    // 5. wrong key entirely (no keyid pin) -> sig fails under wrong pk
    check("signed_unverified (wrong key)", verify(&env, payload, Some(&wrong.pk), None), UNVERIFIED);
    // 6. unsigned envelope, no key configured -> honest unsigned state
    let unsigned = format!("{{\"payloadType\":\"{pt}\",\"payload\":\"{}\",\"signatures\":[{{\"keyid\":\"\",\"sig\":\"\"}}]}}", STANDARD.encode(payload));
    let env_u: Envelope = serde_json::from_str(&unsigned).unwrap();
    check("unsigned_no_key_configured", verify(&env_u, payload, None, None), UNSIGNED);

    let _ = test_kid;
    let _ = wrong_kid;
    if fails == 0 {
        eprintln!("kobold-attest selftest: all 6 states correct");
        0
    } else {
        eprintln!("kobold-attest selftest: {fails} failure(s)");
        1
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("selftest");
    match cmd {
        "selftest" => std::process::exit(selftest()),
        "keygen" => {
            // keygen <seed-hex> -> prints "<pk-hex> <keyid>"; deterministic test keys only.
            let kp = keypair_from_seed_hex(args.get(2).expect("seed hex"));
            println!("{}", hex::encode(kp.pk.as_ref()));
        }
        "sign" => {
            // sign <payload-file> <payloadType> <seed-hex>
            let payload = std::fs::read(args.get(2).expect("payload file")).expect("read payload");
            let pt = args.get(3).expect("payloadType");
            let kp = keypair_from_seed_hex(args.get(4).expect("seed hex"));
            print!("{}", sign(&payload, pt, &kp));
        }
        "verify" => {
            // verify <envelope-file> <expected-payload-file> [<pk-hex> [<keyid>]]  -> prints status, exit 0
            let env: Envelope = serde_json::from_slice(&std::fs::read(args.get(2).expect("envelope")).unwrap()).expect("envelope json");
            let expected = std::fs::read(args.get(3).expect("expected payload")).expect("expected");
            let pk = args.get(4).filter(|s| !s.is_empty()).map(|h| pk_from_hex(h));
            let kid = args.get(5).map(String::as_str);
            println!("{}", verify(&env, &expected, pk.as_ref(), kid));
        }
        _ => {
            eprintln!("usage: kobold-attest selftest | keygen <seed> | sign <payload> <type> <seed> | verify <env> <expected> [pk] [keyid]");
            std::process::exit(2);
        }
    }
}
