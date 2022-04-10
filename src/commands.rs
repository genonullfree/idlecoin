use crate::*;

pub fn read_inputs(
    connections: &Arc<Mutex<Vec<Connection>>>,
    wallets: &Arc<Mutex<Vec<Wallet>>>,
    msg: &mut Vec<String>,
) {
    let mut cons = connections.lock().unwrap();

    for c in cons.iter_mut() {
        let mut buf = [0; 1024];
        let len = match c.stream.read(&mut buf) {
            Ok(l) => l,
            Err(_) => continue,
        };

        if len > 0 {
            println!(
                "> User: 0x{:016x} Miner 0x{:08x} sent: {:?} from: {:?}",
                c.miner.wallet_id,
                c.miner.miner_id,
                &buf[..len],
                c.stream
            );
        }

        let mut new_boost = 0;
        let mut new_boost_cost = 0u128;
        let mut new_miners = 0;
        let mut new_miners_cost = 0u128;

        for i in buf[..len].iter() {
            if *i == b'b' {
                // Purchase boost
                let mut wals = wallets.lock().unwrap();
                for w in wals.iter_mut() {
                    if w.id == c.miner.wallet_id {
                        let cost = buy_boost(c, w);
                        if cost > 0 {
                            new_boost += 128;
                            new_boost_cost += cost as u128;
                        }
                    }
                }
                drop(wals);
            }
            if *i == b'm' {
                // Purchase miner licenses
                let mut wals = wallets.lock().unwrap();
                for w in wals.iter_mut() {
                    if w.id == c.miner.wallet_id {
                        let cost = buy_miner(c, w);
                        if cost > 0 {
                            new_miners += 1;
                            new_miners_cost += cost as u128;
                        }
                    }
                }
                drop(wals);
            }
        }
        let t: DateTime<Local> = Local::now();
        if new_boost > 0 {
            msg.insert(
                0,
                format!(
                    " [{}] Miner 0x{:08x} bought {} boost seconds with {} idlecoin\n",
                    t.format("%Y-%m-%d %H:%M:%S"),
                    c.miner.miner_id,
                    new_boost,
                    new_boost_cost,
                ),
            );
        }
        if new_miners > 0 {
            msg.insert(
                0,
                format!(
                    " [{}] Wallet 0x{:016x} bought new miner license(s) with {} idlecoin\n",
                    t.format("%Y-%m-%d %H:%M:%S"),
                    c.miner.wallet_id,
                    new_miners_cost
                ),
            );
        }
    }
    drop(cons);
}

pub fn boost_cost(cps: u64) -> u64 {
    let v = u64::BITS - cps.leading_zeros() - 1;
    1u64.checked_shl(v).unwrap_or(0)
}

fn buy_boost(connection: &mut Connection, wallet: &mut Wallet) -> u64 {
    if connection.miner.cps < 1024 {
        connection
            .updates
            .push("You need at least 1024 Cps to be able to purchase boost\n".to_string());
        return 0;
    }
    if connection.miner.boost > u16::MAX as u64 {
        connection
            .updates
            .push("You cannot purchase any more boost right now\n".to_string());
        return 0;
    }
    let cost = boost_cost(connection.miner.cps);
    let boost = 128u64;
    if wallet.sub_idlecoins(cost).is_err() {
        connection
            .updates
            .push("You do not have the funds to purchase boost\n".to_string());
        return 0;
    };
    connection.miner.boost = connection.miner.boost.saturating_add(boost);

    cost
}

pub fn miner_cost(max_miners: u64) -> u64 {
    if max_miners < 5 {
        u64::MAX
    } else {
        u64::MAX / (0x100000 >> (max_miners - 5))
    }
}

fn buy_miner(connection: &mut Connection, mut wallet: &mut Wallet) -> u64 {
    if wallet.max_miners >= ABS_MAX_MINERS {
        connection
            .updates
            .push("You cannot purchase any more miners\n".to_string());
        return 0;
    }

    let cost = miner_cost(wallet.max_miners);
    if wallet.sub_idlecoins(cost).is_err() {
        connection
            .updates
            .push("You do not have the funds to purchase a miner\n".to_string());
        return 0;
    };
    wallet.max_miners += 1;

    cost
}
