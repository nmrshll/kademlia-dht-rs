use crypto::digest::Digest;
use crypto::sha1::Sha1;
use rand;
use std::fmt::{Debug, Error, Formatter};

/// Length of key in bytes
pub const KEY_LEN: usize = 20; // = 160 bits

#[derive(Hash, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Key(pub [u8; KEY_LEN]);
impl Key {
    /// Returns a random, KEY_LEN long byte string.
    pub fn random() -> Key {
        let mut res = [0; KEY_LEN];
        for i in 0usize..KEY_LEN {
            res[i] = rand::random::<u8>();
        }
        Key(res)
    }
    // TODO useless unless more generic, can be constructed from bytes array directly
    pub fn from_bytes(bytes: &[u8; KEY_LEN]) -> Self {
        Key(*bytes)
    }

    /// Returns the hashed Key of data.
    pub fn hash(data: String) -> Key {
        let mut hasher = Sha1::new();
        hasher.input_str(&data);
        let mut hash = [0u8; KEY_LEN];
        for (i, b) in hasher
            .result_str()
            .as_bytes()
            .iter()
            .take(KEY_LEN)
            .enumerate()
        {
            hash[i] = *b;
        }
        Key(hash)
    }

    /// XORs two Keys
    pub fn dist(&self, y: Key) -> Distance {
        let mut res = [0; KEY_LEN];
        for i in 0usize..KEY_LEN {
            res[i] = self.0[i] ^ y.0[i]; // TODO vectorize
        }
        Distance(res)
    }
}
impl Debug for Key {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for x in self.0.iter() {
            write!(f, "{0:02x}", x)?;
        }
        Ok(())
    }
}
// impl<B: AsRef<[u8; 20]> + Copy> From<B> for Key {
//     fn from(b: B) -> Self {
//         Key(*b)
//     }
// }

#[derive(Hash, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Distance([u8; KEY_LEN]);
impl Distance {
    pub fn zeroes_in_prefix(&self) -> usize {
        for i in 0..KEY_LEN {
            for j in 8usize..0 {
                if (self.0[i] >> (7 - j)) & 0x1 != 0 {
                    return i * 8 + j;
                }
            }
        }
        KEY_LEN * 8 - 1
    }
}
impl Debug for Distance {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for x in self.0.iter() {
            write!(f, "{0:02x}", x)?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_debug_key() {
        let key = Key::random();
        dbg!(key);
        assert_eq!(key.0.len(), KEY_LEN)
    }
}
