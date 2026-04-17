//! PC-020: Direction enum serde roundtrip test.

use ecc_domain::workflow::transition::Direction;

mod direction {
    use super::*;

    #[test]
    fn serde_roundtrip() {
        // Forward serializes as "forward"
        let forward_json = serde_json::to_string(&Direction::Forward).unwrap();
        assert_eq!(forward_json, r#""forward""#);

        // Backward serializes as "backward"
        let backward_json = serde_json::to_string(&Direction::Backward).unwrap();
        assert_eq!(backward_json, r#""backward""#);

        // Roundtrip Forward
        let restored: Direction = serde_json::from_str(&forward_json).unwrap();
        assert_eq!(restored, Direction::Forward);

        // Roundtrip Backward
        let restored: Direction = serde_json::from_str(&backward_json).unwrap();
        assert_eq!(restored, Direction::Backward);
    }
}
