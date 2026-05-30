use bitcoin::util::address;
use {super::*, clap::ValueEnum};

#[derive(Default, ValueEnum, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Chain {
  #[default]
  #[clap(alias("main"))]
  Mainnet,
  #[clap(alias("test"))]
  Testnet,
  Signet,
  Regtest,
}

impl Chain {
  pub(crate) fn network(self) -> Network {
    match self {
      Self::Mainnet => Network::Bitcoin,
      Self::Testnet => Network::Testnet,
      Self::Signet => Network::Signet,
      Self::Regtest => Network::Regtest,
    }
  }

  pub(crate) fn default_rpc_port(self) -> u16 {
    match self {
      Self::Mainnet => 33873,
      Self::Regtest => 18332,
      Self::Signet => 38332,
      Self::Testnet => 44873,
    }
  }

  pub(crate) fn inscription_content_size_limit(self) -> Option<usize> {
    match self {
      Self::Mainnet | Self::Regtest => None,
      Self::Testnet | Self::Signet => None,
    }
  }

  pub(crate) fn first_inscription_height(self) -> u32 {
    match self {
      Self::Mainnet => 186920,
      Self::Regtest => 0,
      Self::Signet => 0,
      Self::Testnet => 34237,
    }
  }

  pub(crate) fn first_dune_height(self) -> u32 {
    match self {
      Self::Mainnet => 186920,
      Self::Regtest => 0,
      Self::Signet => 0,
      Self::Testnet => 34237,
    }
  }

  pub(crate) fn genesis_block(self) -> Block {
    let genesis_hex: &str = match self {
      Self::Mainnet => "0100000000000000000000000000000000000000000000000000000000000000000000001265bca4002feac94c0c06971f12aa8b2c82fb3e93244690d5cb399aa51b2ad2a01daf65f0ff0f1eb48506000101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff5104ffff001d01044957534a20312f32322f3234202d204665642052657669657720436c656172732043656e7472616c2042616e6b204f6666696369616c73206f662056696f6c6174696e672052756c6573ffffffff010058850c0200000043410436d04f40a76a1094ea10b14a513b62bfd0b47472dda1c25aa9cf8266e53f3c4353680146177f8a3b328ed2c6e02f2b8e051d9d5ffc61a4e6ccabd03409109a5aac00000000",
      Self::Regtest => "0100000000000000000000000000000000000000000000000000000000000000000000001265bca4002feac94c0c06971f12aa8b2c82fb3e93244690d5cb399aa51b2ad2dae5494dffff7f20000000000101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff5104ffff001d01044957534a20312f32322f3234202d204665642052657669657720436c656172732043656e7472616c2042616e6b204f6666696369616c73206f662056696f6c6174696e672052756c6573ffffffff010058850c0200000043410436d04f40a76a1094ea10b14a513b62bfd0b47472dda1c25aa9cf8266e53f3c4353680146177f8a3b328ed2c6e02f2b8e051d9d5ffc61a4e6ccabd03409109a5aac00000000",
      Self::Signet => "010000000000000000000000000000000000000000000000000000000000000000000000696ad20e2dd4365c7459b4a4a5af743d5e92c6da3229e6532cd605f6533f2a5bb9a7f052f0ff0f1ef7390f000101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff1004ffff001d0104084e696e746f6e646fffffffff010058850c020000004341040184710fa689ad5023690c80f3a49c8f13f8d45b8c857fbcbc8bc4a8e4d3eb4b10f4d4604fa08dce601aaf0f470216fe1b51850b4acf21b179c45070ac7b03a9ac00000000",
      Self::Testnet => "0100000000000000000000000000000000000000000000000000000000000000000000001265bca4002feac94c0c06971f12aa8b2c82fb3e93244690d5cb399aa51b2ad2dc1daf65f0ff0f1e940d00000101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff5104ffff001d01044957534a20312f32322f3234202d204665642052657669657720436c656172732043656e7472616c2042616e6b204f6666696369616c73206f662056696f6c6174696e672052756c6573ffffffff010058850c0200000043410436d04f40a76a1094ea10b14a513b62bfd0b47472dda1c25aa9cf8266e53f3c4353680146177f8a3b328ed2c6e02f2b8e051d9d5ffc61a4e6ccabd03409109a5aac00000000",
    };
    let genesis_buf: Vec<u8> = hex::decode(genesis_hex).unwrap();
    bitcoin::consensus::deserialize(&genesis_buf).unwrap()
  }

  pub(crate) fn address_from_script(self, script: &Script) -> Result<Address, address::Error> {
    Address::from_script(script, self.network())
  }

  pub(crate) fn join_with_data_dir(self, data_dir: &Path) -> PathBuf {
    match self {
      Self::Mainnet => data_dir.to_owned(),
      Self::Testnet => data_dir.join("testnet3"),
      Self::Signet => data_dir.join("signet"),
      Self::Regtest => data_dir.join("regtest"),
    }
  }
}

impl Display for Chain {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Mainnet => "mainnet",
        Self::Regtest => "regtest",
        Self::Signet => "signet",
        Self::Testnet => "testnet",
      }
    )
  }
}
