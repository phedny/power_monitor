use std::ops::{AddAssign, MulAssign};
use std::fmt;
use nom::is_digit;

#[derive(Debug,PartialEq)]
pub struct ObisIdentifier {
    a: Option<u8>,
    b: Option<u8>,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
}

fn buf_to_int<T>(s: &[u8]) -> T
where
	T: AddAssign + MulAssign + From<u8>,
{
	let mut sum = T::from(0);
	for digit in s {
		sum *= T::from(10);
		sum += T::from(*digit - b'0');
	}
	sum
}

named!(value_group <&[u8], u8>, map!(take_while_m_n!(1, 3, is_digit), buf_to_int));
named!(value_group_a_delimiter, tag!("-"));
named!(value_group_a <&[u8], u8>, do_parse!(
	value: value_group >>
	_delimiter: value_group_a_delimiter >>
	(value)
));
named!(value_group_b_delimiter, tag!(":"));
named!(value_group_b <&[u8], u8>, do_parse!(
	value: value_group >>
	_delimiter: value_group_b_delimiter >>
	(value)
));
named!(value_group_other_delimiter, tag!("."));
named!(value_group_other <&[u8], u8>, do_parse!(
	value: value_group >>
	_delimiter: value_group_other_delimiter >>
	(value)
));
named!(value_group_f <&[u8], u8>, do_parse!(
	_delimiter: value_group_other_delimiter >>
	value: value_group >>
	(value)
));
named!(pub obis_identifier <&[u8], ObisIdentifier>, do_parse!(
	a: opt!(value_group_a) >>
	b: opt!(value_group_b) >>
	c: value_group_other >>
	d: value_group_other >>
	e: value_group >>
	f: opt!(value_group_f) >>
	(ObisIdentifier { a, b, c, d, e, f: f.unwrap_or(255u8) })
));

impl ObisIdentifier {
	pub fn parse(id: &str) -> Option<ObisIdentifier> {
		match obis_identifier(id.as_bytes()) {
			Ok((_, id)) => Some(id),
			Err(_) => None,
		}
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
