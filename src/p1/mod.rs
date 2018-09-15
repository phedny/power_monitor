extern crate crc;

use std::str;
use self::crc::{crc16, Hasher16};

pub mod reader;

#[derive(Debug, PartialEq)]
pub enum ReadDatagram {
    Datagram(Box<[u8]>),
    IncompleteDatagram(Box<[u8]>),
    InvalidCrc {
    	datagram: Box<[u8]>,
    	expected_crc: Option<u16>,
    	actual_crc: u16,
    },
}

pub fn verify_crc(datagram: ReadDatagram) -> ReadDatagram {
    match datagram {
        ReadDatagram::Datagram(data) => verify_datagram_crc(data),
        x @ _ => x,
    }
}

fn verify_datagram_crc(datagram: Box<[u8]>) -> ReadDatagram {
    let (actual_crc, expected_crc) = {
        let data = &datagram[..datagram.len() - 4];
        let mut digest = crc16::Digest::new_custom(crc16::USB, 0u16, 0u16, crc::CalcType::Reverse);
        digest.write(data);
        let actual_crc = digest.sum16();
        
        let expected_crc = &datagram[datagram.len() - 4..];
        let expected_crc = parse_crc_text(expected_crc);
        (actual_crc, expected_crc)
    };

    match expected_crc {
        Some(expected_crc) if expected_crc == actual_crc => ReadDatagram::Datagram(datagram),
        _ => ReadDatagram::InvalidCrc { datagram, expected_crc, actual_crc }
    }
}

fn parse_crc_text(crc: &[u8]) -> Option<u16> {
    if let Ok(crc) = str::from_utf8(crc) {
        if let Ok(crc) = u16::from_str_radix(crc, 16) {
            Some(crc)
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_remove_the_crc_of_a_correct_datagram() {
        let correct_datagram_1: &[u8] = include_bytes!("correct_datagram_1.test");
        let mut datagram = Vec::new();
        datagram.extend_from_slice(correct_datagram_1);

        let output = verify_datagram_crc(datagram.into_boxed_slice());
        
        let mut expected_datagram = Vec::new();
        expected_datagram.extend_from_slice(correct_datagram_1);
        assert_eq!(output, ReadDatagram::Datagram(expected_datagram.into_boxed_slice()));
    }

    #[test]
    fn it_should_signal_an_invalid_crc_when_the_crc_is_invalid() {
        let correct_datagram_1: &[u8] = include_bytes!("correct_datagram_1.test");
        let mut datagram = Vec::new();
        datagram.extend_from_slice(correct_datagram_1);
        datagram[100] = 15;

        let output = verify_datagram_crc(datagram.to_owned().into_boxed_slice());
        
        let expected_output = ReadDatagram::InvalidCrc {
            datagram: datagram.into_boxed_slice(),
            actual_crc: 0xBAD7,
            expected_crc: Some(0xE47C),
        };
        assert_eq!(output, expected_output);
    }

}
