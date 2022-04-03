use crate::*;

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
                    sub_idlecoins(w, coins);
                    msg.insert(
                        0,
                        format!(
                            " [{}] Wallet 0x{:016x} was taxed 10% by the IRS!\n",
                            t.format("%Y-%m-%d %H:%M:%S"),
                            c.miner.miner_id
                        ),
                    );
                }
            }
            drop(wal);
        } else if x % 10000 == 0 {
            // 0.01 % chance
            let level = c.miner.level;
            dec_level(&mut c.miner);
            if level != c.miner.level {
                msg.insert(
                    0,
                    format!(
                        " [{}] Miner 0x{:08x} lost a level\n",
                        t.format("%Y-%m-%d %H:%M:%S"),
                        c.miner.miner_id
                    ),
                );
            }
        } else if x % 10000 <= 2 {
            // 0.02 % chance
            let level = c.miner.level;
            inc_level(&mut c.miner);
            if level != c.miner.level {
                msg.insert(
                    0,
                    format!(
                        " [{}] Miner 0x{:08x} leveled up\n",
                        t.format("%Y-%m-%d %H:%M:%S"),
                        c.miner.miner_id
                    ),
                );
            };
        } else if x % 10000 <= 3 {
            // .01 % chance
            c.miner.cps += c.miner.cps.saturating_div(10);
            msg.insert(
                0,
                format!(
                    " [{}] Miner 0x{:08x} gained 10% CPS boost\n",
                    t.format("%Y-%m-%d %H:%M:%S"),
                    c.miner.miner_id
                ),
            );
        }
    }

    if msg.len() > 5 {
        msg.resize(5, "".to_owned());
    };

    drop(cons);
}

pub fn process_miners(connections: &Arc<Mutex<Vec<Connection>>>, wallets: &Arc<Mutex<Vec<Wallet>>>) {
    let mut cons = connections.lock().unwrap();
    let mut wals = wallets.lock().unwrap();

    for c in cons.iter_mut() {
        // Update miner
        miner_session(&mut c.miner);
        // Update appropriate wallet
        for w in wals.iter_mut() {
            if c.miner.wallet_id == w.id {
                add_idlecoins(w, c.miner.cps);
            }
        }
    }
    drop(wals);
    drop(cons);
}

fn inc_level(miner: &mut Miner) {
    miner.level = miner.level.saturating_add(1);

    miner.inc = miner.inc.saturating_add(miner.level);

    miner.pow = miner.pow.saturating_mul(10);
}

fn dec_level(miner: &mut Miner) {
    miner.level = miner.level.saturating_sub(1);

    miner.inc = miner.inc.saturating_sub(miner.level);

    miner.pow = miner.pow.saturating_div(10);
}

pub fn add_idlecoins(mut wallet: &mut Wallet, new: u64) {
    wallet.idlecoin = match wallet.idlecoin.checked_add(new) {
        Some(c) => c,
        None => {
            wallet.supercoin = wallet.supercoin.saturating_add(1);
            let x: u128 = (u128::from(wallet.idlecoin) + u128::from(new)) % u128::from(u64::MAX);
            x as u64
        }
    };
}

pub fn sub_idlecoins(mut wallet: &mut Wallet, less: u64) {
    wallet.idlecoin = match wallet.idlecoin.checked_sub(less) {
        Some(c) => c,
        None => {
            if wallet.supercoin > 0 {
                wallet.supercoin = wallet.supercoin.saturating_sub(1);
                (u128::from(u64::MAX) - u128::from(less) + u128::from(wallet.idlecoin))
                    .try_into()
                    .unwrap()
            } else {
                0
            }
        }
    };
}

pub fn miner_session(mut miner: &mut Miner) {
    // Level up
    if miner.cps >= miner.pow {
        inc_level(miner);
    }

    // Increment cps
    let increase = if miner.boost > 0 {
        miner.boost -= 1;
        (miner.inc + miner.level) * 3
    } else {
        miner.inc + miner.level
    };
    miner.cps = miner.cps.saturating_add(increase);
}
