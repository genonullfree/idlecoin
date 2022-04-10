use crate::*;

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Wallet {
    pub id: u64,         // wallet address ID
    pub supercoin: u64,  // supercoin
    pub idlecoin: u64,   // idlecoin
    pub chronocoin: u64, // chronocoin
    pub randocoin: u64,  // randocoin
    pub max_miners: u64, // max number of miners
}

impl Wallet {
    pub fn new(id: u64) -> Wallet {
        Wallet {
            id,
            supercoin: 0,
            idlecoin: 0,
            chronocoin: 0,
            randocoin: 0,
            max_miners: 5,
        }
    }

}
