use std::io;

#[derive(Debug, PartialEq)]
pub enum ReadDatagram {
    Datagram(Vec<u8>),
    IncompleteDatagram(Vec<u8>)
}

pub struct DatagramReader<R> {
    reader: R,
    error: Option<io::Error>
}

impl<R: io::BufRead> DatagramReader<R> {
    pub fn new(reader: R) -> DatagramReader<R> {
        DatagramReader {
            reader,
            error: None
        }
    }

    fn sync_to_datagram(&mut self) -> io::Result<usize> {
        let mut read = 0;
        loop {
            let (available_bytes, dropped_bytes) = {
                let available = self.reader.fill_buf()?;
                (available.len(), available.into_iter().take_while(|b| **b != b'/').count())
            };
            if available_bytes == 0 {
                return Ok(0);
            }
            self.reader.consume(dropped_bytes);
            if dropped_bytes < available_bytes {
                return Ok(read + dropped_bytes);
            }
            read += dropped_bytes;
        }
    }

    fn read_datagram(&mut self) -> io::Result<Vec<u8>> {
        if self.reader.fill_buf()?.len() == 0 {
            return Ok(Vec::new());
        } else {
            self.reader.consume(1);
        }
        let mut datagram = vec![b'/'];
        loop {
            let (available_bytes, read_bytes) = {
                let available = self.reader.fill_buf()?;
                let datagram_bytes = available.iter().take_while(|b| **b != b'/' && **b != b'!').count();
                datagram.extend_from_slice(&available[0..datagram_bytes]);
                (available.len(), datagram_bytes)
            };
            self.reader.consume(read_bytes);
            if available_bytes == 0 || read_bytes < available_bytes {
                return Ok(datagram);
            }
        }
    }

    fn read_crc_bytes(&mut self, datagram: &mut Vec<u8>) -> io::Result<()> {
        if self.reader.fill_buf()?.len() == 0 {
            return Ok(());
        }
        let mut crc_bytes_needed = 5;
        loop {
            let (available_bytes, read_bytes) = {
                let available = self.reader.fill_buf()?;
                let crc_bytes = available.iter().take(crc_bytes_needed).take_while(|b| **b != b'/').count();
                datagram.extend_from_slice(&available[0..crc_bytes]);
                (available.len(), crc_bytes)
            };
            self.reader.consume(read_bytes);
            crc_bytes_needed -= read_bytes;
            if available_bytes == 0 || read_bytes < available_bytes || crc_bytes_needed == 0 {
                return Ok(());
            }
        }
    }

    fn next_datagram(&mut self) -> io::Result<ReadDatagram> {
        let _dropped_bytes = self.sync_to_datagram()?;
        let mut datagram = self.read_datagram()?;
        {
            let available = self.reader.fill_buf()?;
            if available.len() == 0 || available[0] == b'/' {
                return Ok(ReadDatagram::IncompleteDatagram(datagram));
            }
        }
        self.read_crc_bytes(&mut datagram)?;
        if datagram[datagram.len() - 5] == b'!' {
            Ok(ReadDatagram::Datagram(datagram))
        } else {
            Ok(ReadDatagram::IncompleteDatagram(datagram))
        }
    }
}

impl<R: io::BufRead> Iterator for DatagramReader<R> {
    type Item = ReadDatagram;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_datagram() {
            Ok(ReadDatagram::IncompleteDatagram(ref d)) if d.len() == 0 => None,
            Ok(d) => Some(d),
            Err(e) => { self.error = Some(e); None },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct TickleReader<'a> {
        data: &'a [u8],
        ranges: Vec<[usize; 2]>,
    }

    impl<'a> io::Read for TickleReader<'a> {
        fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
            if self.ranges.is_empty() {
                Ok(0)
            } else {
                self.ranges.reverse();
                let r = self.ranges.pop().unwrap();
                let len = r[1] - r[0];
                if len > b.len() {
                    for i in 0..b.len() {
                        b[i] = self.data[i + r[0]];
                    }
                    self.ranges.push([r[0] + b.len(), r[1]]);
                    self.ranges.reverse();
                    Ok(b.len())
                } else {
                    for i in 0..len {
                        b[i] = self.data[i + r[0]];
                    }
                    self.ranges.reverse();
                    Ok(len)
                }
            }
        }
    }

    #[test]
    fn it_should_output_a_single_complete_datagram() {
        let correct_datagram_1: &[u8] = include_bytes!("correct_datagram_1.test");
        let mut reader = DatagramReader::new(io::BufReader::new(correct_datagram_1));

        let datagram = reader.next();

        let mut expected_datagram = Vec::new();
        expected_datagram.extend_from_slice(correct_datagram_1);
        assert_eq!(datagram.unwrap(), ReadDatagram::Datagram(expected_datagram));
    }

    #[test]
    fn it_should_split_an_input_of_two_datagrams_in_two_outputs() {
        let correct_datagram_1: &[u8] = include_bytes!("correct_datagram_1.test");
        let correct_datagram_2: &[u8] = include_bytes!("correct_datagram_2.test");
        let mut combined_input: Vec<u8> = Vec::new();
        combined_input.extend_from_slice(correct_datagram_1);
        combined_input.extend_from_slice(correct_datagram_2);
        let mut reader = DatagramReader::new(io::BufReader::new(combined_input.as_slice()));

        let mut expected_datagram = Vec::new();
        expected_datagram.extend_from_slice(correct_datagram_1);
        let datagram = reader.next();
        assert_eq!(datagram.unwrap(), ReadDatagram::Datagram(expected_datagram));

        let mut expected_datagram = Vec::new();
        expected_datagram.extend_from_slice(correct_datagram_2);
        let datagram = reader.next();
        assert_eq!(datagram.unwrap(), ReadDatagram::Datagram(expected_datagram));
    }

    #[test]
    fn it_should_split_an_input_of_two_datagrams_in_two_outputs_and_ignore_data_in_between_them() {
        let correct_datagram_1: &[u8] = include_bytes!("correct_datagram_1.test");
        let correct_datagram_2: &[u8] = include_bytes!("correct_datagram_2.test");
        let mut combined_input: Vec<u8> = Vec::new();
        combined_input.extend_from_slice(correct_datagram_1);
        combined_input.extend_from_slice(&[4, 23, 32, 55, 75]);
        combined_input.extend_from_slice(correct_datagram_2);
        let mut reader = DatagramReader::new(io::BufReader::new(combined_input.as_slice()));

        let mut expected_datagram = Vec::new();
        expected_datagram.extend_from_slice(correct_datagram_1);
        let datagram = reader.next();
        assert_eq!(datagram.unwrap(), ReadDatagram::Datagram(expected_datagram));

        let mut expected_datagram = Vec::new();
        expected_datagram.extend_from_slice(correct_datagram_2);
        let datagram = reader.next();
        assert_eq!(datagram.unwrap(), ReadDatagram::Datagram(expected_datagram));
    }

    #[test]
    fn it_should_combine_tickling_data_into_a_single_datagram() {
        let correct_datagram_1: &[u8] = include_bytes!("correct_datagram_1.test");
        let tickler = TickleReader {
            data: correct_datagram_1,
            ranges: vec!([0, 5], [5, 17], [17, 19], [19, 44], [44, 544], [544, 763], [763, correct_datagram_1.len()]),
        };
        let mut reader = DatagramReader::new(io::BufReader::with_capacity(1, tickler));

        let datagram = reader.next();

        let mut expected_datagram = Vec::new();
        expected_datagram.extend_from_slice(correct_datagram_1);
        assert_eq!(datagram.unwrap(), ReadDatagram::Datagram(expected_datagram));
    }

    #[test]
    fn it_should_combine_tickling_data_with_a_split_just_before_crc_into_a_single_datagram() {
        let correct_datagram_1: &[u8] = include_bytes!("correct_datagram_1.test");
        let tickler = TickleReader {
            data: correct_datagram_1,
            ranges: vec!([0, 44], [44, correct_datagram_1.len() - 4], [correct_datagram_1.len() - 4, correct_datagram_1.len()]),
        };
        let mut reader = DatagramReader::new(io::BufReader::with_capacity(1, tickler));

        let datagram = reader.next();

        let mut expected_datagram = Vec::new();
        expected_datagram.extend_from_slice(correct_datagram_1);
        assert_eq!(datagram.unwrap(), ReadDatagram::Datagram(expected_datagram));
    }

    #[test]
    fn it_should_combine_tickling_data_with_a_split_in_the_middle_of_crc_into_a_single_datagram() {
        let correct_datagram_1: &[u8] = include_bytes!("correct_datagram_1.test");
        let tickler = TickleReader {
            data: correct_datagram_1,
            ranges: vec!([0, 44], [44, correct_datagram_1.len() - 3], [correct_datagram_1.len() - 3, correct_datagram_1.len()]),
        };
        let mut reader = DatagramReader::new(io::BufReader::with_capacity(1, tickler));

        let datagram = reader.next();

        let mut expected_datagram = Vec::new();
        expected_datagram.extend_from_slice(correct_datagram_1);
        assert_eq!(datagram.unwrap(), ReadDatagram::Datagram(expected_datagram));
    }

    #[test]
    fn it_should_combine_tickling_data_with_a_split_in_the_middle_of_crc_followed_by_trailing_data_into_a_single_datagram() {
        let correct_datagram_1: &[u8] = include_bytes!("correct_datagram_1.test");
        let mut combined_input: Vec<u8> = Vec::new();
        combined_input.extend_from_slice(correct_datagram_1);
        combined_input.extend_from_slice(&[4, 23, 32, 55, 75]);
        let tickler = TickleReader {
            data: correct_datagram_1,
            ranges: vec!([0, 44], [44, correct_datagram_1.len() - 8], [correct_datagram_1.len() - 8, correct_datagram_1.len()]),
        };
        let mut reader = DatagramReader::new(io::BufReader::with_capacity(1, tickler));

        let datagram = reader.next();

        let mut expected_datagram = Vec::new();
        expected_datagram.extend_from_slice(correct_datagram_1);
        assert_eq!(datagram.unwrap(), ReadDatagram::Datagram(expected_datagram));
    }

    #[test]
    fn it_should_signal_an_incomplete_datagram_if_a_datagram_is_truncated_because_a_new_datagram_starts_single_write() {
        let correct_datagram_1: &[u8] = include_bytes!("correct_datagram_1.test");
        let correct_datagram_2: &[u8] = include_bytes!("correct_datagram_2.test");
        let mut combined_input: Vec<u8> = Vec::new();
        combined_input.extend_from_slice(&correct_datagram_1[0..200]);
        combined_input.extend_from_slice(correct_datagram_2);
        let mut reader = DatagramReader::new(io::BufReader::with_capacity(1, combined_input.as_slice()));

        let mut expected_datagram = Vec::new();
        expected_datagram.extend_from_slice(&correct_datagram_1[0..200]);
        let datagram = reader.next();
        assert_eq!(datagram.unwrap(), ReadDatagram::IncompleteDatagram(expected_datagram));

        let mut expected_datagram = Vec::new();
        expected_datagram.extend_from_slice(correct_datagram_2);
        let datagram = reader.next();
        assert_eq!(datagram.unwrap(), ReadDatagram::Datagram(expected_datagram));
    }

    #[test]
    fn it_should_signal_an_incomplete_datagram_if_a_datagram_is_truncated_because_a_new_datagram_starts_separate_writes() {
        let correct_datagram_1: &[u8] = include_bytes!("correct_datagram_1.test");
        let correct_datagram_2: &[u8] = include_bytes!("correct_datagram_2.test");
        let mut combined_input: Vec<u8> = Vec::new();
        combined_input.extend_from_slice(&correct_datagram_1[0..200]);
        combined_input.extend_from_slice(correct_datagram_2);
        let tickler = TickleReader {
            data: combined_input.as_slice(),
            ranges: vec!([0, 200], [200, combined_input.len()]),
        };
        let mut reader = DatagramReader::new(io::BufReader::with_capacity(1, tickler));

        let mut expected_datagram = Vec::new();
        expected_datagram.extend_from_slice(&correct_datagram_1[0..200]);
        let datagram = reader.next();
        assert_eq!(datagram.unwrap(), ReadDatagram::IncompleteDatagram(expected_datagram));

        let mut expected_datagram = Vec::new();
        expected_datagram.extend_from_slice(correct_datagram_2);
        let datagram = reader.next();
        assert_eq!(datagram.unwrap(), ReadDatagram::Datagram(expected_datagram));
    }

    #[test]
    fn it_should_signal_an_incomplete_datagram_if_a_datagram_has_truncated_crc_because_a_new_datagram_starts_separate_writes() {
        let correct_datagram_1: &[u8] = include_bytes!("correct_datagram_1.test");
        let correct_datagram_2: &[u8] = include_bytes!("correct_datagram_2.test");
        let mut combined_input: Vec<u8> = Vec::new();
        combined_input.extend_from_slice(&correct_datagram_1[0..correct_datagram_1.len() - 3]);
        combined_input.extend_from_slice(correct_datagram_2);
        let tickler = TickleReader {
            data: combined_input.as_slice(),
            ranges: vec!([0, correct_datagram_1.len() - 3], [correct_datagram_1.len() - 3, combined_input.len()]),
        };
        let mut reader = DatagramReader::new(io::BufReader::with_capacity(1, tickler));

        let mut expected_datagram = Vec::new();
        expected_datagram.extend_from_slice(&correct_datagram_1[0..correct_datagram_1.len() - 3]);
        let datagram = reader.next();
        assert_eq!(datagram.unwrap(), ReadDatagram::IncompleteDatagram(expected_datagram));

        let mut expected_datagram = Vec::new();
        expected_datagram.extend_from_slice(correct_datagram_2);
        let datagram = reader.next();
        assert_eq!(datagram.unwrap(), ReadDatagram::Datagram(expected_datagram));
    }
}
