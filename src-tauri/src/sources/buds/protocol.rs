const START_OF_MESSAGE: u8 = 0xFD;
const END_OF_MESSAGE: u8 = 0xDD;
const SIZE_MASK: u16 = 0x03FF;
const HEADER_LEN: usize = 3;
const CRC_LEN: usize = 2;
const MIN_BODY_LEN: usize = 1 + CRC_LEN;
const MAX_BUFFERED_BYTES: usize = 4096;
const CRC_POLYNOMIAL: u16 = 0x1021;

pub const STATUS_UPDATED: u8 = 0x60;
pub const EXTENDED_STATUS_UPDATED: u8 = 0x61;
pub const NOISE_CONTROLS_UPDATE: u8 = 0x77;
pub const NOISE_CONTROLS: u8 = 0x78;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub id: u8,
    pub payload: Vec<u8>,
}

pub fn crc16(data: &[u8]) -> u16 {
    data.iter().fold(0u16, |crc, byte| {
        (0..8).fold(crc ^ (u16::from(*byte) << 8), |value, _| {
            if value & 0x8000 != 0 {
                (value << 1) ^ CRC_POLYNOMIAL
            } else {
                value << 1
            }
        })
    })
}

pub fn encode(id: u8, payload: &[u8]) -> Vec<u8> {
    let body_len = (1 + payload.len() + CRC_LEN) as u16 & SIZE_MASK;
    let mut checked = vec![id];
    checked.extend_from_slice(payload);
    let crc = crc16(&checked);
    let mut packet = vec![START_OF_MESSAGE];
    packet.extend_from_slice(&body_len.to_le_bytes());
    packet.extend_from_slice(&checked);
    packet.extend_from_slice(&crc.to_le_bytes());
    packet.push(END_OF_MESSAGE);
    packet
}

#[derive(Default)]
pub struct FrameDecoder {
    buffer: Vec<u8>,
}

impl FrameDecoder {
    pub fn push(&mut self, bytes: &[u8]) -> Vec<Frame> {
        self.buffer.extend_from_slice(bytes);
        let mut frames = Vec::new();
        while let Some(step) = self.next_step() {
            frames.extend(step);
        }
        if self.buffer.len() > MAX_BUFFERED_BYTES {
            self.buffer.clear();
        }
        frames
    }

    fn next_step(&mut self) -> Option<Option<Frame>> {
        let start = self.buffer.iter().position(|byte| *byte == START_OF_MESSAGE)?;
        self.buffer.drain(..start);
        if self.buffer.len() < HEADER_LEN {
            return None;
        }
        let body_len = usize::from(u16::from_le_bytes([self.buffer[1], self.buffer[2]]) & SIZE_MASK);
        if body_len < MIN_BODY_LEN {
            self.buffer.remove(0);
            return Some(None);
        }
        let total_len = HEADER_LEN + body_len + 1;
        if self.buffer.len() < total_len {
            return None;
        }
        let frame = parse_packet(&self.buffer[..total_len], body_len);
        let consumed = if frame.is_some() { total_len } else { 1 };
        self.buffer.drain(..consumed);
        Some(frame)
    }
}

fn parse_packet(packet: &[u8], body_len: usize) -> Option<Frame> {
    if packet.last() != Some(&END_OF_MESSAGE) {
        return None;
    }
    let body = &packet[HEADER_LEN..HEADER_LEN + body_len];
    let (checked, crc_bytes) = body.split_at(body_len - CRC_LEN);
    let expected_crc = u16::from_le_bytes([crc_bytes[0], crc_bytes[1]]);
    (crc16(checked) == expected_crc).then(|| Frame { id: checked[0], payload: checked[1..].to_vec() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_matches_xmodem_check_value() {
        assert_eq!(crc16(b"123456789"), 0x31C3);
    }

    #[test]
    fn encoded_packet_decodes_back() {
        let packet = encode(NOISE_CONTROLS, &[1]);
        let frames = FrameDecoder::default().push(&packet);
        assert_eq!(frames, vec![Frame { id: NOISE_CONTROLS, payload: vec![1] }]);
    }

    #[test]
    fn decoder_resyncs_after_garbage_and_split_reads() {
        let packet = encode(STATUS_UPDATED, &[1, 2, 3]);
        let mut decoder = FrameDecoder::default();
        assert!(decoder.push(&[0x00, 0x13]).is_empty());
        assert!(decoder.push(&packet[..4]).is_empty());
        let frames = decoder.push(&packet[4..]);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].payload, vec![1, 2, 3]);
    }

    #[test]
    fn corrupted_checksum_is_dropped() {
        let mut packet = encode(STATUS_UPDATED, &[9, 9]);
        let crc_index = packet.len() - 2;
        packet[crc_index] ^= 0xFF;
        assert!(FrameDecoder::default().push(&packet).is_empty());
    }
}
