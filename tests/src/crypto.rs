#[cfg(test)]
mod tests {

    use jam_types::OpaqueHash;
    use sp_core::{ed25519, Pair};
    use tools::{{hex, log}, serde::{Deserialize, Value}};
    
    #[derive(Clone, Debug, PartialEq)]
    struct Testcase {
        number: u32,
        desc: String,
        pk: OpaqueHash,
        r: OpaqueHash,
        s: OpaqueHash,
        msg: Vec<u8>,
        pk_canonical: bool,
        r_canonical: bool,
    }

    impl Deserialize for Testcase {
        fn from_value(v: &Value) -> Result<Self, String> {
            let o = match v {
                Value::Object(o) => o,
                _ => return Err("expected object".into()),
            };

            Ok(Testcase{
                number: u32::from_value(o.get("number").ok_or("missing number")?)?,
                desc: String::from_value(o.get("desc").ok_or("missing desc")?)?,
                pk: OpaqueHash::from_value(o.get("pk").ok_or("missing pk")?)?,
                r: OpaqueHash::from_value(o.get("r").ok_or("missing r")?)?,
                s: OpaqueHash::from_value(o.get("s").ok_or("missing s")?)?,
                msg: Vec::<u8>::from_value(o.get("msg").ok_or("missing msg")?)?,
                pk_canonical: bool::from_value(o.get("pk_canonical").ok_or("missing pk_canonical")?)?,
                r_canonical: bool::from_value(o.get("r_canonical").ok_or("missing r_canonical")?)?,
            })
        }
    }

#[test]
fn crypto_ed25519_test() {
    use tools::serde::{from_json_str, Value};
    use std::io::Read;

    log::Builder::from_env(log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();

    let mut path = std::path::PathBuf::from("/home/bernar/workspace/jam-conformance/crypto/ed25519/vectors.json");
    let mut file = std::fs::File::open(&path).expect("Failed to open JSON file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Failed to read JSON file");

    let value: Value = from_json_str(&contents).unwrap();

    fn parse_testcase(v: &Value) -> Result<Testcase, String> {
        let o = match v {
            Value::Object(o) => o,
            _ => return Err("expected object".into()),
        };

        let msg_hex = match o.get("msg") {
            Some(Value::String(s)) => s,
            _ => return Err("missing msg".into()),
        };
        let msg_hex = msg_hex.strip_prefix("0x").unwrap_or(msg_hex);
        let msg = hex::decode(msg_hex).map_err(|_| "invalid msg hex".to_string())?;

        Ok(Testcase {
            number: u32::from_value(o.get("number").ok_or("missing number")?)?,
            desc: String::from_value(o.get("desc").ok_or("missing desc")?)?,
            pk: OpaqueHash::from_value(o.get("pk").ok_or("missing pk")?)?,
            r: OpaqueHash::from_value(o.get("r").ok_or("missing r")?)?,
            s: OpaqueHash::from_value(o.get("s").ok_or("missing s")?)?,
            msg,
            pk_canonical: bool::from_value(o.get("pk_canonical").ok_or("missing pk_canonical")?)?,
            r_canonical: bool::from_value(o.get("r_canonical").ok_or("missing r_canonical")?)?,
        })
    }

    let testcases: Vec<Testcase> = match &value {
        Value::Array(a) => {
            a.iter()
                .map(parse_testcase)
                .collect::<Result<_, _>>()
                .unwrap()
        }
        Value::Object(o) => {
            let arr = o
                .get("vectors")
                .or_else(|| o.get("tests"))
                .or_else(|| o.get("cases"))
                .expect("missing array");
            match arr {
                Value::Array(a) => a.iter().map(parse_testcase).collect::<Result<_, _>>().unwrap(),
                _ => panic!("expected array inside object"),
            }
        }
        _ => panic!("unexpected json top-level"),
    };

    assert!(!testcases.is_empty());

    println!("testcase 2: {:x?}", testcases[1]);

    for tc in testcases {
        let mut sig_bytes = [0u8; 64];
        sig_bytes[..32].copy_from_slice(&tc.r);
        sig_bytes[32..].copy_from_slice(&tc.s);
        let signature = ed25519::Signature::from_raw(sig_bytes);
        let public_key = ed25519::Public::from_raw(tc.pk);

        let ok = ed25519::Pair::verify(&signature, &tc.msg, &public_key);
        let expected = tc.pk_canonical && tc.r_canonical;

        log::info!("tc {} {} -> ok={}, expected={}", tc.number, tc.desc, ok, expected);
        assert_eq!(ok, expected, "mismatch in testcase {}", tc.number);
    }
}



}