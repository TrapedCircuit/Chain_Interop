use ethers::types::U256;

#[derive(Debug, Clone, Default)]
pub struct ZeroCopyWriter {
    pub buf: Vec<u8>,
    pub offset: usize,
}

impl ZeroCopyWriter {
    pub fn from(buf: Vec<u8>) -> Self {
        Self { buf, offset: 0 }
    }

    pub fn write_var_bytes(&mut self, bytes: &[u8]) -> &[u8] {
        self.write_uint(bytes.len() as u64);
        self.write_bytes(bytes)
    }

    pub fn write_u256(&mut self, num: &ethers::types::U256) -> &[u8] {
        let mut bytes = [0; 32];
        num.to_little_endian(&mut bytes);
        self.write_bytes(&bytes)
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> &[u8] {
        self.buf.extend_from_slice(bytes);
        let len = bytes.len();
        let buf_len = self.buf.len();
        &self.buf[buf_len - len..buf_len]
    }

    pub fn write_uint(&mut self, num: u64) {
        match num {
            _ if num < 0xFD => {
                let num = num as u8;
                self.buf.extend_from_slice(&[num]);
            }
            _ if num < 0xFFFF => {
                let buf: Vec<u8> = vec![0xFD];
                let num = (num as u16).to_le_bytes();
                self.buf.extend_from_slice(&buf);
                self.buf.extend_from_slice(&num);
            }
            _ if num < 0xFFFFFFFF => {
                let buf: Vec<u8> = vec![0xFE];
                let num = (num as u32).to_le_bytes();
                self.buf.extend_from_slice(&buf);
                self.buf.extend_from_slice(&num);
            }
            _ => {
                let buf: Vec<u8> = vec![0xFF];
                let num = num.to_le_bytes();
                self.buf.extend_from_slice(&buf);
                self.buf.extend_from_slice(&num);
            }
        }
    }

    pub fn read_next_bytes(&mut self) -> Vec<u8> {
        let len = self.read_len();
        let mut bytes = vec![0; len];
        bytes.copy_from_slice(&self.buf[self.offset..self.offset + len]);
        self.offset += len;

        bytes
    }

    pub fn read_u256(&mut self) -> ethers::types::U256 {
        let mut bytes = [0; 32];
        bytes.copy_from_slice(&self.buf[self.offset..self.offset + 32]);
        ethers::types::U256::from_little_endian(&bytes)
    }

    pub fn read_len(&mut self) -> usize {
        let offset = self.offset;
        match self.buf[offset] {
            x if x < 0xFD => {
                self.offset += 1;
                x as usize
            }
            0xFD => {
                self.offset += 3;
                u16::from_le_bytes(self.buf[offset + 1..offset + 3].try_into().unwrap()) as usize
            }
            0xFE => {
                self.offset += 5;
                u32::from_le_bytes(self.buf[offset + 1..offset + 5].try_into().unwrap()) as usize
            }
            0xFF => {
                self.offset += 9;
                u64::from_le_bytes(self.buf[offset + 1..offset + 9].try_into().unwrap()) as usize
            }
            _ => unreachable!(),
        }
    }
}

pub fn to_payload(to_asset_addr: &str, to_addr: &str, amount: U256) -> Vec<u8> {
    let mut w = ZeroCopyWriter::default();
    w.write_var_bytes(to_asset_addr.as_bytes());
    w.write_var_bytes(to_addr.as_bytes());
    w.write_u256(&amount);
    w.buf
}

pub fn from_payload(payload: &[u8]) -> anyhow::Result<(String, String, U256)> {
    let mut deser = ZeroCopyWriter::from(payload.to_vec());
    let to_asset_addr = String::from_utf8(deser.read_next_bytes())?;
    let to_addr = String::from_utf8(deser.read_next_bytes())?;
    let amount = deser.read_u256();
    Ok((to_asset_addr, to_addr, amount))
}

#[test]
fn test_zero_copy_writer() {
    use ethers::types::U256;
    use std::str::FromStr;

    let mut w = ZeroCopyWriter::default();

    let to_asset_addr = ethers::types::Address::from_str("0xa5A5dC4A6F869e279AC32b1925d2605a96289859").unwrap();
    let to_addr = ethers::types::Address::from_str("0x5CB1fA08AAAF49A9d3C80af80AF177b3035083E0").unwrap();
    let amount = U256::from(100u64);

    w.write_var_bytes(to_asset_addr.as_bytes());
    w.write_var_bytes(to_addr.as_bytes());
    w.write_u256(&amount);

    let buf = w.buf;

    let mut w = ZeroCopyWriter::from(buf);

    let to_asset_addr2 = ethers::types::Address::from_slice(&w.read_next_bytes());
    let to_addr2 = ethers::types::Address::from_slice(&w.read_next_bytes());
    let amount2 = w.read_u256();

    assert_eq!(to_asset_addr, to_asset_addr2);
    assert_eq!(to_addr, to_addr2);
    assert_eq!(amount, amount2);
}
