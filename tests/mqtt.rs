slicewrap::wrap! {
    /// An MQTT compatible UTF-8 string.
    ///
    /// In MQTT may be at most 2^16-1 bytes long and must not
    /// contain internal `\0` bytes
    #[derive(Debug, Hash, Eq, PartialEq)]
    pub struct MqttStr(str), from = [Box, Rc, Arc];
}

#[derive(Debug, Eq, PartialEq)]
pub struct Error;

impl MqttStr {
    const MAX_LEN: usize = u16::MAX as usize;

    pub fn from_str(s: &str) -> Result<&MqttStr, Error> {
        if s.contains('\0') {
            return Err(Error);
        }

        match s.len() {
            0..=Self::MAX_LEN => Ok(MqttStr::from_ref(s)),
            _ => Err(Error),
        }
    }
}

#[test]
fn mqtt_from_str() {
    let m = MqttStr::from_str("Hello, World").unwrap();
    assert_eq!(m, "Hello, World");

    let mut res = MqttStr::from_str("Hello\0World");
    assert_eq!(res, Err(Error));

    let len = 1 << 18;
    let long_string: String = (0..len).map(|_| 'a').collect();
    res = MqttStr::from_str(&long_string);
    assert_eq!(res, Err(Error));
}
