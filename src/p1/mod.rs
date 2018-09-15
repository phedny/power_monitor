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
