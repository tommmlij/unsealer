
#[cfg(test)]
mod service_tests {
    use super::*; // Import items from the parent module (your service code)
    use crate::cli::{Cli, base64key::Base64Key, SecretKeyValue, PublicKeyValue}; // Adjust path as needed
    use sodiumoxide::crypto::box_;
    use std::str::FromStr;
    use std::sync::Arc;
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE;
    use tokio::sync::Mutex;
    use crate::api::{AppState, InitRequest};
    use crate::test_helpers;

    #[tokio::test]
    async fn test_init_handler_success() {
        sodiumoxide::init().ok();
        let (server_pk, server_sk) = box_::gen_keypair();;
        let (manager_pk, manager_sk) = box_::gen_keypair();;

        let server_sk_base = URL_SAFE.encode(server_sk);
        let manager_pk_base = URL_SAFE.encode(manager_pk);

        let app_state = Arc::new(Mutex::new(AppState {
            server_private_key: SecretKeyValue::from_str(&server_sk_base).unwrap(),
            manager_public_key: PublicKeyValue::from_str(&manager_pk_base).unwrap(),
            config: None,
        }));

        let secret_message = "{\"api_key\":\"secret_value\"}";
        let encrypted_config = test_helpers::encrypt_payload(secret_message, &server_pk, &manager_sk);

        let request = InitRequest { config: encrypted_config };
        let result = init_handler(State(app_state.clone()), Json(request)).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Service unsealed and started.");

        let state_guard = app_state.lock().await;
        assert_eq!(state_guard.config.as_deref(), Some(secret_message));

        // Check if notified (can be tricky to assert directly without waiting,
        // but for unit test, checking state change is primary)
    }

    #[tokio::test]
    async fn test_init_handler_decryption_failure() {
        sodiumoxide::init().ok();
        let (_server_pk, server_sk) = test_helpers::generate_keys(); // Correct server sk
        let (manager_pk, _manager_sk) = test_helpers::generate_keys(); // Correct manager pk for CLI

        // Create a different key pair for encryption, simulating wrong client or server key
        let (rogue_server_pk, _rogue_server_sk) = test_helpers::generate_keys();
        let (_rogue_manager_pk, rogue_manager_sk) = test_helpers::generate_keys();


        let cli = create_test_cli(server_sk, manager_pk); // Server configured with its actual sk and expected manager pk
        let shutdown_trigger = Arc::new(Notify::new());
        let app_state = Arc::new(Mutex::new(AppState {
            cli,
            shutdown_trigger,
            config: None,
        }));

        let secret_message = "{\"api_key\":\"secret_value\"}";
        // Encrypt with a key the server isn't expecting for the manager, or for itself
        let encrypted_config = test_helpers::encrypt_payload(secret_message, &rogue_server_pk, &rogue_manager_sk);

        let request = InitRequest { config: encrypted_config };
        let result = init_handler(State(app_state.clone()), Json(request)).await;

        assert!(result.is_err());
        if let Err((status, message)) = result {
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
            assert_eq!(message, "Decryption failed");
        } else {
            panic!("Expected an error result");
        }

        let state_guard = app_state.lock().await;
        assert!(state_guard.config.is_none());
    }

    #[tokio::test]
    async fn test_init_handler_invalid_payload_short() {
        sodiumoxide::init().ok();
        let (_server_pk, server_sk) = test_helpers::generate_keys();
        let (manager_pk, _manager_sk) = test_helpers::generate_keys();

        let cli = create_test_cli(server_sk, manager_pk);
        let shutdown_trigger = Arc::new(Notify::new());
        let app_state = Arc::new(Mutex::new(AppState {
            cli,
            shutdown_trigger,
            config: None,
        }));

        // Nonce alone is typically 24 bytes. This is too short.
        let short_payload = URL_SAFE.encode("short");
        let request = InitRequest { config: short_payload };
        let result = init_handler(State(app_state.clone()), Json(request)).await;

        assert!(result.is_err());
        if let Err((status, message)) = result {
            assert_eq!(status, StatusCode::BAD_REQUEST);
            // Message might depend on exact check that fails first
            assert!(message.contains("Invalid encrypted payload") || message.contains("Payload too short"));
        } else {
            panic!("Expected an error result");
        }
    }
    // TODO: Add tests for invalid base64, non-UTF8 decrypted payload (if using the improved error handling)
}