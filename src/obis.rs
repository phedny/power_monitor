use std::fmt;
use regex::Regex;

#[derive(Debug,PartialEq)]
pub struct ObisIdentifier {
    a: Option<u8>,
    b: Option<u8>,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
}

impl ObisIdentifier {
	pub fn parse(id: &str) -> Option<ObisIdentifier> {
		lazy_static! {
			static ref ID_PATTERN: Regex = Regex::new(r"^(?:(\d+)-)?(?:(\d+):)?(\d+)\.(\d+)(?:\.(\d+))?(?:\.(\d+))?$").unwrap();
		}

		ID_PATTERN.captures(id).map(|cap| {
			let a = cap.get(1).map(|g| u8::from_str_radix(g.as_str(), 10).unwrap());
			let b = cap.get(2).map(|g| u8::from_str_radix(g.as_str(), 10).unwrap());
			let c = cap.get(3).map(|g| u8::from_str_radix(g.as_str(), 10).unwrap()).unwrap();
			let d = cap.get(4).map(|g| u8::from_str_radix(g.as_str(), 10).unwrap()).unwrap();
			let e = cap.get(5).map(|g| u8::from_str_radix(g.as_str(), 10).unwrap()).unwrap();
			let f = cap.get(6).map_or(255u8, |g| u8::from_str_radix(g.as_str(), 10).unwrap());

			ObisIdentifier { a, b, c, d, e, f }
		})
	}
}

impl fmt::Display for ObisIdentifier {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if let Some(a) = self.a {
			write!(f, "{}-", a)?;
		}
		if let Some(b) = self.b {
			write!(f, "{}:", b)?;
		}
		write!(f, "{}.{}.{}.{}", self.c, self.d, self.e, self.f)
	}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_convert_to_string_id_with_all_value_groups() {
        let id = ObisIdentifier { a: Some(1), b: Some(0), c: 96, d: 7, e: 21, f: 255 };
        assert_eq!(id.to_string(), "1-0:96.7.21.255");
    }

    #[test]
    fn it_should_parse_string_with_all_value_groups() {
    	let id = ObisIdentifier::parse("1-0:96.7.21.255").unwrap();
    	assert_eq!(id, ObisIdentifier { a: Some(1), b: Some(0), c: 96, d: 7, e: 21, f: 255 });
    }

    #[test]
    fn it_should_convert_to_string_id_without_value_group_a() {
        let id = ObisIdentifier { a: None, b: Some(0), c: 96, d: 7, e: 21, f: 255 };
        assert_eq!(id.to_string(), "0:96.7.21.255");
    }

    #[test]
    fn it_should_parse_string_without_value_group_a() {
    	let id = ObisIdentifier::parse("0:96.7.21.255").unwrap();
    	assert_eq!(id, ObisIdentifier { a: None, b: Some(0), c: 96, d: 7, e: 21, f: 255 });
    }

    #[test]
    fn it_should_convert_to_string_id_without_value_group_b() {
        let id = ObisIdentifier { a: Some(1), b: None, c: 96, d: 7, e: 21, f: 255 };
        assert_eq!(id.to_string(), "1-96.7.21.255");
    }

    #[test]
    fn it_should_parse_string_without_value_group_b() {
    	let id = ObisIdentifier::parse("1-96.7.21.255").unwrap();
    	assert_eq!(id, ObisIdentifier { a: Some(1), b: None, c: 96, d: 7, e: 21, f: 255 });
    }

    #[test]
    fn it_should_convert_to_string_id_without_value_groups_a_and_b() {
        let id = ObisIdentifier { a: None, b: None, c: 96, d: 7, e: 21, f: 255 };
        assert_eq!(id.to_string(), "96.7.21.255");
    }

    #[test]
    fn it_should_parse_string_without_value_groups_a_and_b() {
    	let id = ObisIdentifier::parse("96.7.21.255").unwrap();
    	assert_eq!(id, ObisIdentifier { a: None, b: None, c: 96, d: 7, e: 21, f: 255 });
    }

    #[test]
    fn it_should_parse_string_without_value_group_f() {
    	let id = ObisIdentifier::parse("1-0:96.7.21").unwrap();
    	assert_eq!(id, ObisIdentifier { a: Some(1), b: Some(0), c: 96, d: 7, e: 21, f: 255 });
    }

}
