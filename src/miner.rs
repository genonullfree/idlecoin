use crate::*;

#[derive(Copy, Clone, Debug)]
pub struct Miner {
    pub miner_id: u32,  // miner address ID
    pub wallet_id: u64, // wallet address ID
    pub level: u64,     // current level
    pub cps: u64,       // coin-per-second
    inc: u64,           // Incrementor value
    pow: u64,           // Next level up value
    pub boost: u64,     // Seconds of boosted cps
}

pub fn action_miners(
    connections: &Arc<Mutex<Vec<Connection>>>,
    wallets: &Arc<Mutex<Vec<Wallet>>>,
    msg: &mut Vec<String>,
) {
    let mut rng = rand::thread_rng();

    let mut cons = connections.lock().unwrap();

    for c in cons.iter_mut() {
        let t: DateTime<Local> = Local::now();
        let x: u64 = rng.gen();

        if x % 15552000 == 0 {
            // 0.00000006430041152263 % chance
            let mut wal = wallets.lock().unwrap();
            for w in wal.iter_mut() {
                if w.id == c.miner.wallet_id {
                    w.supercoin -= w.supercoin.saturating_div(10);
                    let coins = w.idlecoin.saturating_div(10);
                    if w.sub_idlecoins(coins).is_err() {
                        continue;
                    };
                    msg.insert(
                        0,
                        format!(
                            " [{}] Wallet {}0x{:016x}{} was taxed 10% by the IRS!\n",
                            t.format("%Y-%m-%d %H:%M:%S"),
                            BLUE,
                            c.miner.miner_id,
                            RST,
                        ),
                    );
                }
            }
            drop(wal);
        } else if x % 100000 == 0 {
            // 0.001 % chance
            let level = c.miner.level;
            c.miner.dec_level();
            if level != c.miner.level {
                msg.insert(
                    0,
                    format!(
                        " [{}] Miner {}0x{:08x}{} lost a level\n",
                        t.format("%Y-%m-%d %H:%M:%S"),
                        BLUE,
                        c.miner.miner_id,
                        RST,
                    ),
                );
            }
        } else if x % 100000 <= 2 {
            // 0.002 % chance
            let level = c.miner.level;
            c.miner.inc_level();
            if level != c.miner.level {
                msg.insert(
                    0,
                    format!(
                        " [{}] Miner {}0x{:08x}{} leveled up\n",
                        t.format("%Y-%m-%d %H:%M:%S"),
                        BLUE,
                        c.miner.miner_id,
                        RST,
                    ),
                );
            };
        } else if x % 100000 <= 3 {
            // .001 % chance
            c.miner.cps += c.miner.cps.saturating_div(10);
            msg.insert(
                0,
                format!(
                    " [{}] Miner {}0x{:08x}{} gained 10% CPS boost\n",
                    t.format("%Y-%m-%d %H:%M:%S"),
                    BLUE,
                    c.miner.miner_id,
                    RST,
                ),
            );
        }
    }

    drop(cons);
}

pub fn process_miners(
    connections: &Arc<Mutex<Vec<Connection>>>,
    wallets: &Arc<Mutex<Vec<Wallet>>>,
) {
    let mut cons = connections.lock().unwrap();
    let mut wals = wallets.lock().unwrap();

    for c in cons.iter_mut() {
        // Update miner
        miner_session(&mut c.miner);
        // Update appropriate wallet
        for w in wals.iter_mut() {
            if c.miner.wallet_id == w.id {
                w.add_idlecoins(c.miner.cps);
            }
        }
    }
    drop(wals);
    drop(cons);
}

impl Miner {
    pub fn new(wallet_id: u64, miner_id: u32) -> Miner {
        Miner {
            miner_id,
            wallet_id,
            level: 0,
            cps: 0,
            inc: 1,
            pow: 10,
            boost: 0,
        }
    }

    pub fn inc_level(&mut self) {
        self.level = self.level.saturating_add(1);
        self.inc = self.inc.saturating_add(self.level);
        self.pow = self.pow.saturating_mul(10);
    }

    pub fn dec_level(&mut self) {
        self.level = self.level.saturating_sub(1);
        self.inc = self.inc.saturating_sub(self.level);
        self.pow = self.pow.saturating_div(10);
    }

    pub fn print(&self) -> String {
        format!(
            "[M:{}0x{:0>8x}{} Cps:{}{}{} B:{} L:{:<2}] ",
            BLUE,
            self.miner_id,
            RST,
            GREEN,
            utils::disp_units(self.cps),
            RST,
            utils::disp_units(self.boost),
            self.level,
        )
    }
}

pub fn miner_session(mut miner: &mut Miner) {
    // Level up
    if miner.cps >= miner.pow {
        miner.inc_level();
    }

    // Increment cps
    miner.inc = if miner.boost > 0 {
        miner.boost -= 1;
        miner.inc.saturating_add(miner.level * miner.level * 3)
    } else {
        miner.inc.saturating_add(miner.level * miner.level)
    };
    miner.cps = miner.cps.saturating_add(miner.inc);
}
