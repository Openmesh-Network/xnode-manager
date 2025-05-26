use tiny_keccak::{Hasher, Keccak};

// From: https://github.com/gakonst/ethers-rs/blob/master/ethers-core/src/utils/hash.rs

/// Hash a message according to [EIP-191] (version `0x01`).
///
/// The final message is a UTF-8 string, encoded as follows:
/// `"\x19Ethereum Signed Message:\n" + message.length + message`
///
/// This message is then hashed using [Keccak-256](keccak256).
///
/// [EIP-191]: https://eips.ethereum.org/EIPS/eip-191
pub fn hash_message<T: AsRef<[u8]>>(message: T) -> [u8; 32] {
    const PREFIX: &str = "\x19Ethereum Signed Message:\n";

    let message = message.as_ref();
    let len = message.len();
    let len_string = len.to_string();

    let mut eth_message = Vec::with_capacity(PREFIX.len() + len_string.len() + len);
    eth_message.extend_from_slice(PREFIX.as_bytes());
    eth_message.extend_from_slice(len_string.as_bytes());
    eth_message.extend_from_slice(message);

    keccak256(&eth_message)
}

/// Compute the Keccak-256 hash of input bytes.
///
/// Note that strings are interpreted as UTF-8 bytes,
fn keccak256<T: AsRef<[u8]>>(bytes: T) -> [u8; 32] {
    let mut output = [0u8; 32];

    let mut hasher = Keccak::v256();
    hasher.update(bytes.as_ref());
    hasher.finalize(&mut output);

    output
}
