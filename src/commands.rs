use crate::*;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash)]
enum PurchaseType {
    Boost,
    Miner,
}

#[derive(Copy, Clone, PartialEq, Hash)]
struct Purchase {
    bought: usize,
    cost: u128,
}

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

        // Server terminal output
        if len > 0 {
            println!(
                "> User: 0x{:016x} Miner 0x{:08x} sent: {:?} from: {:?}",
                c.miner.wallet_id,
                c.miner.miner_id,
                &buf[..len],
                c.stream
            );
        } else {
            continue;
        }

        let mut upgrades = HashMap::new();

        // Iterate through each char in the received buffer
        'nextchar: for i in buf[..len].iter() {
            if *i == b'b' {
                // Purchase boost
                let mut wals = wallets.lock().unwrap();
                for w in wals.iter_mut() {
                    if w.id == c.miner.wallet_id {
                        let cost = match buy_boost(c, w) {
                            Ok(c) => c,
                            Err(e) => {
                                c.updates.push(e.to_string());
                                continue 'nextchar;
                            }
                        };
                        let new = Purchase {
                            bought: 128,
                            cost: cost as u128,
                        };
                        update_upgrade_list(&mut upgrades, PurchaseType::Boost, new);
                    }
                }
                drop(wals);
            }
            if *i == b'm' {
                // Purchase miner licenses
                let mut wals = wallets.lock().unwrap();
                for w in wals.iter_mut() {
                    if w.id == c.miner.wallet_id {
                        let cost = match buy_miner(w) {
                            Ok(c) => c,
                            Err(e) => {
                                c.updates.push(e.to_string());
                                continue 'nextchar;
                            }
                        };
                        let new = Purchase {
                            bought: 1,
                            cost: cost as u128,
                        };
                        update_upgrade_list(&mut upgrades, PurchaseType::Miner, new);
                    }
                }
                drop(wals);
            }
        }

        // Purchase notification updates
        for (k, p) in upgrades.iter() {
            let t: DateTime<Local> = Local::now();
            match k {
                PurchaseType::Boost => msg.insert(
                    0,
                    format!(
                        " [{}] Miner 0x{:08x} bought {} boost seconds with {} idlecoin\n",
                        t.format("%Y-%m-%d %H:%M:%S"),
                        c.miner.miner_id,
                        p.bought,
                        p.cost,
                    ),
                ),
                PurchaseType::Miner => msg.insert(
                    0,
                    format!(
                        " [{}] Wallet 0x{:016x} bought {} new miner license(s) with {} idlecoin\n",
                        t.format("%Y-%m-%d %H:%M:%S"),
                        c.miner.wallet_id,
                        p.bought,
                        p.cost,
                    ),
                ),
            }
        }
    }
    drop(cons);
}

fn update_upgrade_list(
    map: &mut HashMap<PurchaseType, Purchase>,
    p_type: PurchaseType,
    p: Purchase,
) {
    let mut node = match map.get(&p_type) {
        Some(n) => *n,
        None => Purchase { bought: 0, cost: 0 },
    };
    node.bought += p.bought;
    node.cost += p.cost;
    map.insert(p_type, node);
}

pub fn boost_cost(cps: u64) -> u64 {
    let v = u64::BITS - cps.leading_zeros() - 1;
    1u64.checked_shl(v).unwrap_or(0)
}

fn buy_boost(connection: &mut Connection, wallet: &mut Wallet) -> Result<u64, Error> {
    if connection.miner.cps < 1024 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "You need at least 1024 Cps to be able to purchase boost\n",
        ));
    }
    if connection.miner.boost > u16::MAX as u64 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "You cannot purchase any more boost right now\n",
        ));
    }
    let cost = boost_cost(connection.miner.cps);
    if wallet.sub_idlecoins(cost).is_err() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "You do not have the funds to purchase boost\n",
        ));
    }
    connection.miner.boost = connection.miner.boost.saturating_add(128);

    Ok(cost)
}

pub fn miner_cost(max_miners: u64) -> u64 {
    if max_miners < 5 {
        u64::MAX
    } else {
        u64::MAX / (0x100000 >> (max_miners - 5))
    }
}

fn buy_miner(mut wallet: &mut Wallet) -> Result<u64, Error> {
    if wallet.max_miners >= ABS_MAX_MINERS {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "You cannot purchase any more miners\n",
        ));
    }

    let cost = miner_cost(wallet.max_miners);
    if wallet.sub_idlecoins(cost).is_err() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "You do not have the funds to purchase a miner\n",
        ));
    };
    wallet.max_miners += 1;

    Ok(cost)
}
