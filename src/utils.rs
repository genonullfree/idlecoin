pub fn disp_units(num: u64) -> String {
    let unit = [' ', 'K', 'M', 'G', 'T', 'P', 'E', 'Z', 'Y'];
    let mut value = num as f64;

    let mut count = 0;
    loop {
        if (value / 1000.0) > 1.0 {
            count += 1;
            value /= 1000.0;
        } else {
            break;
        }
        if count == unit.len() - 1 {
            break;
        }
    }

    let n = if count > 0 { 1 } else { 0 };
    format!("{:.*}{:>1}", n, value, unit[count])
}
