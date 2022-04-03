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

        for i in buf[..len].iter() {
            // TODO: Rework buying
            if *i == 98 {
                // 'b'
                let mut wals = wallets.lock().unwrap();
                for w in wals.iter_mut() {
                    if w.id == c.miner.wallet_id {
                        if w.idlecoin < 1024 && w.supercoin < 1 {
                            c.updates.push(
                                "You need at least 1024 idlecoin to be able to purchase boost\n"
                                    .to_string(),
                            );
                            continue;
                        }
                        let cost = 1024u64;
                        let boost = 128u64;
                        let t: DateTime<Local> = Local::now();
                        msg.insert(
                            0,
                            format!(
                                " [{}] Miner 0x{:08x} bought {} boost seconds with {} idlecoin\n",
                                t.format("%Y-%m-%d %H:%M:%S"),
                                c.miner.miner_id,
                                boost,
                                cost
                            ),
                        );
                        miner::sub_idlecoins(w, cost);
                        c.miner.boost = c.miner.boost.saturating_add(boost);
                    }
                }
                drop(wals);
            }
            if *i == 109 {
                // 'm'
                let mut wals = wallets.lock().unwrap();
                for w in wals.iter_mut() {
                    if w.id == c.miner.wallet_id {
                        if w.max_miners >= 10 {
                            c.updates
                                .push("You cannot purchase any more miners\n".to_string());
                            continue;
                        }
                        let cost = u64::MAX / (100000 >> (w.max_miners - 5));
                        if w.idlecoin > cost || w.supercoin > 0 {
                            let t: DateTime<Local> = Local::now();
                            msg.insert(0, format!(" [{}] Wallet 0x{:016x} bought a new miner license with {} idlecoin\n", t.format("%Y-%m-%d %H:%M:%S"), c.miner.wallet_id, cost));
                            miner::sub_idlecoins(w, cost);
                            w.max_miners += 1;
                        } else {
                            c.updates.push(format!(
                                "You need {} idlecoin to purchase another miner license\n",
                                cost
                            ));
                        }
                    }
                }
                drop(wals);
            }
        }
    }
    drop(cons);
}
