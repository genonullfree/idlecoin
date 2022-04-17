use crate::*;
use std::collections::HashMap;
use std::ops::AddAssign;

#[derive(PartialEq, Eq, Hash)]
enum PurchaseType {
    Boost,
    Miner,
    Chrono,
}

#[derive(Copy, Clone, PartialEq, Hash)]
struct Purchase {
    bought: usize,
    cost: u128,
}

impl Purchase {
    fn new() -> Self {
        Purchase { bought: 0, cost: 0 }
    }
}

impl AddAssign for Purchase {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            bought: self.bought + other.bought,
            cost: self.cost + other.cost,
        };
    }
}

pub fn read_inputs(
    connections: &Arc<Mutex<Vec<Connection>>>,
    wallets: &Arc<Mutex<Vec<Wallet>>>,
    msg: &mut Vec<String>,
) {
    let mut cons = connections.lock().unwrap();

    for c in cons.iter_mut() {
        let mut buf = [0; 4096];
        let mut upgrades = HashMap::new();
        while let Ok(len) = c.stream.read(&mut buf) {
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
                break;
            }

            let mut wals = wallets.lock().unwrap();
            for w in wals.iter_mut() {
                if w.id == c.miner.wallet_id {
                    // Iterate through each char in the received buffer
                    for i in buf[..len].iter() {
                        let new = match *i {
                            b'b' => {
                                // Purchase boost
                                match buy_boost(c, w) {
                                    Ok(c) => Some(c),
                                    Err(e) => {
                                        if !c.updates.iter().any(|u| *u == e.to_string()) {
                                            c.updates.push(e.to_string());
                                        }
                                        continue;
                                    }
                                }
                            }
                            b'B' => {
                                // Purchase MAX boost
                                buy_max_boost(c, w)
                            }
                            b'm' => {
                                // Purchase miner licenses
                                match buy_miner(w) {
                                    Ok(c) => Some(c),
                                    Err(e) => {
                                        if !c.updates.iter().any(|u| *u == e.to_string()) {
                                            c.updates.push(e.to_string());
                                        }
                                        continue;
                                    }
                                }
                            }
                            b'M' => {
                                // Purchase MAX miners
                                buy_max_miner(w)
                            }
                            b'c' => {
                                // Purchase time travel
                                match buy_time(c, w) {
                                    Ok(c) => Some(c),
                                    Err(e) => {
                                        if !c.updates.iter().any(|u| *u == e.to_string()) {
                                            c.updates.push(e.to_string());
                                        }
                                        continue;
                                    }
                                }
                            }
                            b'C' => {
                                // Purchase MAX time travel
                                buy_max_time(c, w)
                            }
                            _ => None,
                        };
                        if let Some((p_type, p)) = new {
                            update_upgrade_list(&mut upgrades, p_type, p);
                        };
                    }
                }
            }
        }

        // Purchase notification updates
        for (k, p) in upgrades.iter() {
            let t: DateTime<Local> = Local::now();

            // Skip if nothing was purchased
            if p.cost == 0 {
                continue;
            }

            match k {
                PurchaseType::Boost => msg.insert(
                    0,
                    format!(
                        " [{}] Miner {}0x{:08x}{} bought {} boost seconds with {}{}{} idlecoin\n",
                        t.format("%Y-%m-%d %H:%M:%S"), BLUE,
                        c.miner.miner_id, RST,
                        p.bought, YELLOW,
                        p.cost,RST,
                    ),
                ),
                PurchaseType::Miner => msg.insert(
                    0,
                    format!(
                        " [{}] Wallet {}0x{:016x}{} bought {} new miner license(s) with {}{}{} idlecoin\n",
                        t.format("%Y-%m-%d %H:%M:%S"), BLUE,
                        c.miner.wallet_id, RST,
                        p.bought,YELLOW,
                        p.cost,RST,
                    ),
                ),
                PurchaseType::Chrono => msg.insert(
                    0,
                    format!(" [{}] Miner {}0x{:08x}{} travelled {} hours forward in time with {}{}{} chronocoins\n",
                        t.format("%Y-%m-%d %H:%M:%S"),BLUE,
                        c.miner.miner_id,RST,
                        p.bought,YELLOW,
                        p.cost,RST,
                    )),
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
    node += p;
    map.insert(p_type, node);
}

pub fn boost_cost(cps: u64) -> u64 {
    let v = u64::BITS - cps.leading_zeros() - 1;
    1u64.checked_shl(v).unwrap_or(0)
}

pub fn boost_max_cost(cps: u64, boost: u64) -> u64 {
    let cost = boost_cost(cps);
    let left = (u16::MAX as u64 - boost) / 128;
    cost * left
}

fn buy_boost(
    connection: &mut Connection,
    wallet: &mut Wallet,
) -> Result<(PurchaseType, Purchase), Error> {
    if connection.miner.cps < 1024 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "You need at least 1024 Cps to be able to purchase boost\n",
        ));
    }
    if connection.miner.boost >= u16::MAX as u64 - 128 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "You cannot purchase any more boost right now\n",
        ));
    }
    let cost = boost_cost(connection.miner.cps);
    if wallet.sub_idlecoins(cost).is_err() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("You need {} idlecoins to purchase boost\n", cost),
        ));
    }
    let bought: usize = 128;
    connection.miner.boost = connection.miner.boost.saturating_add(bought as u64);
    if connection.miner.boost > u16::MAX as u64 {
        connection.miner.boost = u16::MAX as u64;
    }

    Ok((
        PurchaseType::Boost,
        Purchase {
            bought,
            cost: cost as u128,
        },
    ))
}

fn buy_max_boost(
    connection: &mut Connection,
    wallet: &mut Wallet,
) -> Option<(PurchaseType, Purchase)> {
    let mut totals = Purchase::new();

    loop {
        let (_, p) = match buy_boost(connection, wallet) {
            Ok(b) => b,
            Err(_) => return Some((PurchaseType::Boost, totals)),
        };

        totals += p;
    }
}

pub fn miner_cost(max_miners: u64) -> u64 {
    if max_miners < 5 {
        u64::MAX
    } else {
        u64::MAX / (0x100000 >> (max_miners - 5))
    }
}

pub fn miner_max_cost(max_miners: u64) -> u128 {
    let mut max = max_miners;
    let mut cost = 0u128;
    loop {
        if max <= ABS_MAX_MINERS {
            cost += miner_cost(max) as u128;
            max += 1;
        } else {
            break;
        }
    }
    cost
}

fn buy_miner(mut wallet: &mut Wallet) -> Result<(PurchaseType, Purchase), Error> {
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
            format!("You need {} idlecoins to purchase a miner\n", cost),
        ));
    };
    wallet.max_miners += 1;

    Ok((
        PurchaseType::Miner,
        Purchase {
            bought: 1,
            cost: cost as u128,
        },
    ))
}

fn buy_max_miner(wallet: &mut Wallet) -> Option<(PurchaseType, Purchase)> {
    let mut totals = Purchase::new();

    loop {
        let (_, p) = match buy_miner(wallet) {
            Ok(m) => m,
            Err(_) => return Some((PurchaseType::Miner, totals)),
        };

        totals += p;
    }
}

pub fn time_cost() -> u64 {
    1000
}

pub fn time_max_cost(chrono: u64) -> u64 {
    (chrono / 1000) * 1000
}

fn buy_time(
    connection: &mut Connection,
    wallet: &mut Wallet,
) -> Result<(PurchaseType, Purchase), Error> {
    if wallet.sub_chronocoins(time_cost()).is_err() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "You need 1000 chronocoins to purchase a time travel\n",
        ));
    }

    let mut c = 60 * 60;
    loop {
        miner_session(&mut connection.miner);
        c -= 1;
        if c == 0 {
            break;
        }
    }

    Ok((
        PurchaseType::Chrono,
        Purchase {
            bought: 1,
            cost: time_cost() as u128,
        },
    ))
}

fn buy_max_time(
    connection: &mut Connection,
    wallet: &mut Wallet,
) -> Option<(PurchaseType, Purchase)> {
    let mut totals = Purchase::new();

    loop {
        let (_, p) = match buy_time(connection, wallet) {
            Ok(t) => t,
            Err(_) => return Some((PurchaseType::Chrono, totals)),
        };

        totals += p;
    }
}
