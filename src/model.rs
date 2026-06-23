use crate::ffi::{AttackType, CNonce, MF_CLASSIC_KEY_SIZE};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MfClassicKey {
    pub data: [u8; MF_CLASSIC_KEY_SIZE],
}

impl MfClassicKey {
    pub fn from_slice(s: &[u8]) -> Self {
        let mut data = [0u8; MF_CLASSIC_KEY_SIZE];
        data.copy_from_slice(&s[..MF_CLASSIC_KEY_SIZE]);
        MfClassicKey { data }
    }

    pub fn to_hex(&self) -> String {
        self.data.iter().map(|b| format!("{:02X}", b)).collect()
    }
}

#[derive(Debug, Clone)]
pub struct Nonce {
    pub attack: AttackType,
    pub uid: u32,
    pub nt0: u32,
    pub nt1: u32,
    pub uid_xor_nt0: u32,
    pub uid_xor_nt1: u32,

    pub ks1_1_enc: u32,
    pub ks1_2_enc: u32,
    pub par_1: u8,
    pub par_2: u8,

    pub nr0_enc: u32,
    pub ar0_enc: u32,
    pub nr1_enc: u32,
    pub ar1_enc: u32,
    pub p64: u32,
    pub p64b: u32,
}

impl Nonce {
    pub fn to_c(&self) -> CNonce {
        CNonce {
            attack: self.attack as i32,
            uid: self.uid,
            nt0: self.nt0,
            nt1: self.nt1,
            uid_xor_nt0: self.uid_xor_nt0,
            uid_xor_nt1: self.uid_xor_nt1,
            ks1_1_enc: self.ks1_1_enc,
            ks1_2_enc: self.ks1_2_enc,
            par_1: self.par_1,
            par_2: self.par_2,
            p64: self.p64,
            p64b: self.p64b,
            nr0_enc: self.nr0_enc,
            ar0_enc: self.ar0_enc,
            nr1_enc: self.nr1_enc,
            ar1_enc: self.ar1_enc,
        }
    }

    pub fn attack_name(&self) -> &'static str {
        match self.attack {
            AttackType::StaticNested => "static_nested",
            AttackType::StaticEncrypted => "static_encrypted",
            AttackType::Mfkey32 => "mfkey32",
        }
    }
}

impl Default for Nonce {
    fn default() -> Self {
        Nonce {
            attack: AttackType::Mfkey32,
            uid: 0,
            nt0: 0,
            nt1: 0,
            uid_xor_nt0: 0,
            uid_xor_nt1: 0,
            ks1_1_enc: 0,
            ks1_2_enc: 0,
            par_1: 0,
            par_2: 0,
            nr0_enc: 0,
            ar0_enc: 0,
            nr1_enc: 0,
            ar1_enc: 0,
            p64: 0,
            p64b: 0,
        }
    }
}
