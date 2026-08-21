//! nt_io_desktop — 桌面端 IO 能力 (更新器签名等)
//! 契约: updater_keygen / updater_pubkey / updater_sign

pub mod updater_signing;

pub use updater_signing::{
    UpdaterSigner, SignOp, KeyPair, PublicKeyInfo, SigningOutput, SignatureData,
    SigningError, CliInput, CliOutput, execute_signing,
};