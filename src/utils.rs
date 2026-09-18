use sha1::{Sha1, Digest};



pub fn get_sha1_info_hash(info: &Vec<u8>) ->Vec<u8> {

    let mut hasher = Sha1::new();
    hasher.update(info);
    let info_hash_sha1: sha1::digest::generic_array::GenericArray<u8, sha1::digest::typenum::UInt<sha1::digest::typenum::UInt<sha1::digest::typenum::UInt<sha1::digest::typenum::UInt<sha1::digest::typenum::UInt<sha1::digest::typenum::UTerm, sha1::digest::consts::B1>, sha1::digest::consts::B0>, sha1::digest::consts::B1>, sha1::digest::consts::B0>, sha1::digest::consts::B0>> = hasher.finalize();
    info_hash_sha1.to_vec()
}

pub fn percent_encode(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|b| format!("%{:02X}", b))
        .collect()
}


