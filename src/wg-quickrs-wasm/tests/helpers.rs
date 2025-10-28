use wg_quickrs_wasm::helpers::*;
use wg_quickrs_wasm::types::conf::WireGuardKey;

struct TestVector<'a> {
    priv_b64: &'a str,
    expected_pub_b64: &'a str,
}

#[test]
fn test_wireguard_vectors() {
    // generated using 'wg genkey' and 'echo priv_key | wg pubkey'
    let vectors = [
        TestVector {
            priv_b64: "wODITqX4oJtjT1N4Mx17K2dRaogd9i/ZBhgNVsVoDlg=",
            expected_pub_b64: "QMUUFvCRL9SVEBmnM1or3lC7VZI/pLgnN9jrPIJngzk=",
        },
        TestVector {
            priv_b64: "QOp247ORpfEimcZFE4DxS5+LTJ5s5mUfpiqTnaEtC0Q=",
            expected_pub_b64: "uo2hR4Jw2lv/0+db23XgnE32jN5woDKWUHACft7W/Eo=",
        },
        TestVector {
            priv_b64: "CG4OFx+dbGpGi0wgHmiF36wzZvfIHyJ03M4KtOsms0Q=",
            expected_pub_b64: "wzTKDk4Ws8z58okzyLrJTnN5tkAJmBBOwTIq52RzgWQ=",
        },
        TestVector {
            priv_b64: "0O70e+Qb6Icf+/uvw6r6MUxBri6VAnNmxyQo8sQrklU=",
            expected_pub_b64: "iWHLk867Z0LTBDL319noIlFuxo7A1GAZVN6rWc6XKHE=",
        },
        TestVector {
            priv_b64: "mN2aMFu+/T9STqwjOu3ZyVs2PZizpCPX80T5BXhrQ2w=",
            expected_pub_b64: "ojX+ccUrzjzJ7PgHecHmSW2SL/3Go7UUgr8p+WyFnnA=",
        },
    ];

    for vec in vectors {
        let derived = wg_public_key_from_private_key(&WireGuardKey::from_base64(vec.priv_b64).unwrap());
        assert_eq!(
            derived, WireGuardKey::from_base64(vec.expected_pub_b64).unwrap(),
            "Mismatch for private key: {}",
            vec.priv_b64
        );
    }
}
