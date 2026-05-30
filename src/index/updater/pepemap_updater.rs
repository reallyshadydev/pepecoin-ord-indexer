use super::*;
use crate::drc20::operation::{Action, InscriptionOp};
use crate::drc20::script_key::ScriptKey;
use crate::inscription::Inscription;
use crate::pepemap::{self, PepemapEntry};
use bitcoin::Transaction;
use std::collections::HashMap;

pub(crate) struct PepemapContext {
  pub network: Network,
  pub block_height: u32,
  pub block_time: u32,
}

pub(super) struct PepemapUpdater<'a, 'db, 'tx> {
  number_to_entry: &'a mut Table<'db, 'tx, u32, &'static [u8]>,
  inscription_to_number: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, u32>,
  owner_to_numbers: &'a mut MultimapTable<'db, 'tx, &'static [u8], u32>,
  transaction_id_to_transaction: &'a mut Table<'db, 'tx, &'static TxidValue, &'static [u8]>,
}

impl<'a, 'db, 'tx> PepemapUpdater<'a, 'db, 'tx> {
  pub fn new(
    number_to_entry: &'a mut Table<'db, 'tx, u32, &'static [u8]>,
    inscription_to_number: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, u32>,
    owner_to_numbers: &'a mut MultimapTable<'db, 'tx, &'static [u8], u32>,
    transaction_id_to_transaction: &'a mut Table<'db, 'tx, &'static TxidValue, &'static [u8]>,
  ) -> Result<Self> {
    Ok(Self {
      number_to_entry,
      inscription_to_number,
      owner_to_numbers,
      transaction_id_to_transaction,
    })
  }

  pub fn index_block(
    &mut self,
    context: PepemapContext,
    block: &BlockData,
    operations: HashMap<Txid, Vec<InscriptionOp>>,
  ) -> Result {
    let tx_lookup: HashMap<Txid, &Transaction> =
      block.txdata.iter().map(|(tx, txid)| (*txid, tx)).collect();

    for (txid, tx_operations) in operations {
      for op in tx_operations {
        match &op.action {
          Action::New { inscription } => {
            self.handle_claim(&context, &tx_lookup, txid, &op, inscription)?
          }
          Action::Transfer => self.handle_transfer(&context, &tx_lookup, txid, &op)?,
        }
      }
    }

    Ok(())
  }

  fn handle_claim(
    &mut self,
    context: &PepemapContext,
    tx_lookup: &HashMap<Txid, &Transaction>,
    txid: Txid,
    op: &InscriptionOp,
    inscription: &Inscription,
  ) -> Result {
    let Some(number) = pepemap::parse_number(inscription) else {
      return Ok(());
    };

    if number > context.block_height {
      // Cannot pre-claim future blocks.
      return Ok(());
    }

    if self.number_to_entry.get(&number)?.is_some() {
      // Already claimed.
      return Ok(());
    }

    let new_satpoint = match op.new_satpoint {
      Some(point) => point,
      None => return Ok(()),
    };

    let owner = self.resolve_owner(tx_lookup, new_satpoint, context.network)?;
    let owner_key = owner.to_string();

    let entry = PepemapEntry {
      number,
      inscription_id: op.inscription_id,
      txid,
      owner: owner_key.clone(),
      block_height: context.block_height,
      block_time: context.block_time,
    };

    let encoded = bincode::serialize(&entry)?;
    self.number_to_entry.insert(&number, encoded.as_slice())?;
    self
      .inscription_to_number
      .insert(&op.inscription_id.store(), &number)?;
    self
      .owner_to_numbers
      .insert(owner_key.as_bytes(), &number)?;

    Ok(())
  }

  fn handle_transfer(
    &mut self,
    context: &PepemapContext,
    tx_lookup: &HashMap<Txid, &Transaction>,
    txid: Txid,
    op: &InscriptionOp,
  ) -> Result {
    let Some(number) = self
      .inscription_to_number
      .get(&op.inscription_id.store())?
      .map(|value| value.value())
    else {
      return Ok(());
    };

    let stored_entry = self
      .number_to_entry
      .get(&number)?
      .ok_or_else(|| anyhow!("pepemap entry missing for claimed number {number}"))?;
    let mut entry: PepemapEntry = bincode::deserialize(stored_entry.value())?;
    drop(stored_entry);

    let new_satpoint = match op.new_satpoint {
      Some(point) => point,
      None => return Ok(()),
    };

    let new_owner = self.resolve_owner(tx_lookup, new_satpoint, context.network)?;

    let new_owner_key = new_owner.to_string();

    if entry.owner == new_owner_key {
      return Ok(());
    }

    let old_owner_key = entry.owner.clone();
    self
      .owner_to_numbers
      .remove(old_owner_key.as_bytes(), &number)?;

    entry.owner = new_owner_key.clone();
    entry.txid = txid;
    entry.block_height = context.block_height;
    entry.block_time = context.block_time;

    let encoded = bincode::serialize(&entry)?;
    self.number_to_entry.insert(&number, encoded.as_slice())?;
    self
      .owner_to_numbers
      .insert(new_owner_key.as_bytes(), &number)?;

    Ok(())
  }

  fn resolve_owner(
    &self,
    tx_lookup: &HashMap<Txid, &Transaction>,
    satpoint: SatPoint,
    network: Network,
  ) -> Result<ScriptKey> {
    if let Some(tx) = tx_lookup.get(&satpoint.outpoint.txid) {
      let script = tx.output[satpoint.outpoint.vout as usize]
        .script_pubkey
        .clone();
      return Ok(ScriptKey::from_script(&script, network));
    }

    if let Some(raw_tx) = self
      .transaction_id_to_transaction
      .get(&satpoint.outpoint.txid.store())?
    {
      let tx: Transaction = consensus::encode::deserialize(raw_tx.value())?;
      let script = tx.output[satpoint.outpoint.vout as usize]
        .script_pubkey
        .clone();
      return Ok(ScriptKey::from_script(&script, network));
    }

    Err(anyhow!(
      "failed to resolve owner for satpoint {}",
      satpoint.outpoint
    ))
  }
}
