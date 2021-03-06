use crate::*;

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Wallet {
    pub id: u64, // wallet address ID
    #[serde(default = "def_zero")]
    pub supercoin: u64, // supercoin
    #[serde(default = "def_zero")]
    pub idlecoin: u64, // idlecoin
    #[serde(default = "def_zero")]
    pub chronocoin: u64, // chronocoin
    #[serde(default = "def_zero")]
    pub randocoin: u64, // randocoin
    #[serde(default = "def_five")]
    pub max_miners: u64, // max number of miners
}

fn def_zero() -> u64 {
    0u64
}

fn def_five() -> u64 {
    5u64
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

    pub fn inc_chronocoins(&mut self) {
        self.chronocoin = self.chronocoin.saturating_add(1);
    }

    pub fn sub_chronocoins(&mut self, less: u64) -> Result<(), Error> {
        self.chronocoin = match self.chronocoin.checked_sub(less) {
            Some(c) => c,
            None => return Err(Error::new(ErrorKind::InvalidData, "Not enough chronocoin")),
        };
        Ok(())
    }

    pub fn inc_randocoins(&mut self) {
        self.randocoin = self.randocoin.saturating_add(16);
    }

    pub fn sub_randocoins(&mut self, less: u64) -> Result<(), Error> {
        self.randocoin = match self.randocoin.checked_sub(less) {
            Some(c) => c,
            None => return Err(Error::new(ErrorKind::InvalidData, "Not enough randocoin")),
        };
        Ok(())
    }

    pub fn print(&self) -> String {
        format!("Wallet {}0x{:016x}{} Miner Licenses: {}{}{} Chronocoin: {}{}{} Randocoin: {}{}{} Coins: {}{}:{}{}",
            PURPLE, self.id, RST,
            BLUE, self.max_miners, RST,
            YELLOW, self.chronocoin, RST,
            YELLOW, self.randocoin, RST,
            YELLOW, self.supercoin, self.idlecoin, RST,
        )
    }
}
