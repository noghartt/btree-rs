pub fn bool_to_byte(b: bool) -> u8 {
  if b { 0x01 } else { 0x00 }
}

pub fn byte_to_bool(b: u8) -> bool {
  b == 0x01
}
