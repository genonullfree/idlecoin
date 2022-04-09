use crate::*;

pub fn load_stats(wallets: &Arc<Mutex<Vec<Wallet>>>) -> Result<(), Error> {
    let mut j = String::new();

    // Attempt to open and read the saved stats file
    let mut file = match File::open(&SAVE) {
        Ok(f) => f,
        Err(_) => {
            println!("No stats file found.");
            return Ok(());
        }
    };

    file.read_to_string(&mut j)?;

    // Exit if file is empty
    if j.is_empty() {
        return Err(Error::new(ErrorKind::InvalidData, "No data to load"));
    }

    // Attempt to deserialize the json file data
    println!("Loading stats...");
    if let Ok(mut wallet) = serde_json::from_str(&j) {
        // Update the wallets struct
        let mut gens = wallets.lock().unwrap();
        gens.append(&mut wallet);
        drop(gens);
        println!("Successfully loaded stats file {}", SAVE);
    } else {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Failed to load {}", SAVE),
        ));
    }

    Ok(())
}

pub fn save_stats(wallets: &Arc<Mutex<Vec<Wallet>>>) {
    // Serialize the stats data to json
    println!("Saving stats...");
    let gens = wallets.lock().unwrap();
    let j = serde_json::to_string_pretty(&gens.deref()).unwrap();
    drop(gens);

    // Open the stats file for writing
    let mut file = match File::create(&SAVE) {
        Ok(f) => f,
        Err(_) => {
            println!("Error opening {} for writing!", SAVE);
            return;
        }
    };

    // Write out the json stats data
    let len = file.write(j.as_bytes()).unwrap();
    if j.len() != len {
        println!("Error writing save data to {}", SAVE);
        return;
    }

    println!("Successfully saved data to {}", SAVE);
}
