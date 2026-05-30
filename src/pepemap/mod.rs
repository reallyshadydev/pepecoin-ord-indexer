use crate::{Inscription, InscriptionId};
use bitcoin::Txid;
use serde::{Deserialize, Serialize};

pub const SUFFIX: &str = ".pepemap";
pub const MAX_NUMBER: u32 = 5_000_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PepemapEntry {
  pub number: u32,
  pub inscription_id: InscriptionId,
  pub txid: Txid,
  pub owner: String,
  pub block_height: u32,
  pub block_time: u32,
}

pub fn parse_number(inscription: &Inscription) -> Option<u32> {
  let body = inscription.body()?;
  let body_str = std::str::from_utf8(body).ok()?.trim();
  if body_str.is_empty() {
    return None;
  }

  let (number_part, suffix) = body_str.rsplit_once('.')?;
  if !suffix.eq_ignore_ascii_case(SUFFIX.trim_start_matches('.')) {
    return None;
  }

  let number_str = number_part.trim();
  if number_str.is_empty() {
    return None;
  }

  let number = number_str.parse::<u32>().ok()?;
  if number == 0 || number > MAX_NUMBER {
    return None;
  }

  Some(number)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_number_supports_uppercase_suffix() {
    let inscription = Inscription::new(None, Some(b"123.PEPEMAP".to_vec()));
    assert_eq!(parse_number(&inscription), Some(123));
  }

  #[test]
  fn parse_number_trims_whitespace() {
    let inscription = Inscription::new(None, Some(b" 456 .pepemap \n".to_vec()));
    assert_eq!(parse_number(&inscription), Some(456));
  }

  #[test]
  fn parse_number_rejects_invalid_suffix() {
    let inscription = Inscription::new(None, Some(b"789.invalid".to_vec()));
    assert_eq!(parse_number(&inscription), None);
  }
}
