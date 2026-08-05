// crates/geargab-core/tests/error_tests.rs

use geargab_core::error::GearGabError;

#[test]
fn test_invalid_osc_address_formatting() {
    let err = GearGabError::InvalidOscAddress("/invalid/path#bad".to_string());
    assert_eq!(
        err.to_string(),
        "Invalid OSC address path: /invalid/path#bad"
    );
}

#[test]
fn test_missing_argument_formatting() {
    let err = GearGabError::MissingArgument {
        expected: "client_uuid",
        index: 0,
    };
    assert_eq!(
        err.to_string(),
        "Missing required OSC argument 'client_uuid' at index 0"
    );
}

#[test]
fn test_type_mismatch_formatting() {
    let err = GearGabError::TypeMismatch {
        expected: "String (s)",
        found: "Int (i)",
        index: 1,
    };
    assert_eq!(
        err.to_string(),
        "OSC argument type mismatch at index 1: expected String (s), found Int (i)"
    );
}

#[test]
fn test_rosc_error_conversion() {
    let rosc_err = rosc::OscError::BadOscAddress("bad_addr".to_string());
    let err: GearGabError = rosc_err.into();
    
    match err {
        GearGabError::OscCodec(inner) => {
            assert!(inner.to_string().contains("bad_addr"));
        }
        _ => panic!("Expected GearGabError::OscCodec variant"),
    }
}

#[test]
fn test_invalid_uuid_formatting() {
    let err = GearGabError::InvalidUuid("not-a-uuid".to_string());
    assert_eq!(err.to_string(), "Invalid UUID format: not-a-uuid");
}