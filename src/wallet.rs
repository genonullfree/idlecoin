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

    pub fn add_idlecoins(&mut self, new: u64) {
        self.idlecoin = match self.idlecoin.checked_add(new) {
            Some(c) => c,
            None => {
                self.supercoin = self.supercoin.saturating_add(1);
                let x: u128 = (u128::from(self.idlecoin) + u128::from(new)) % u128::from(u64::MAX);
                x as u64
            }
        };
    }

    pub fn sub_idlecoins(&mut self, less: u64) -> Result<(), Error> {
        self.idlecoin = match self.idlecoin.checked_sub(less) {
            Some(c) => c,
            None => {
                if self.supercoin > 0 {
                    self.supercoin = self.supercoin.saturating_sub(1);
                    (u128::from(u64::MAX) - u128::from(less) + u128::from(self.idlecoin))
                        .try_into()
                        .unwrap()
                } else {
                    return Err(Error::new(ErrorKind::InvalidData, "Not enough idlecoin"));
                }
            }
        };
        Ok(())
    }

    pub fn add_chronocoins(&mut self, add: u64) {
        self.chronocoin = match self.chronocoin.checked_add(add) {
            Some(c) => c,
            None => u64::MAX,
        }
    }
}
