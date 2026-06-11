use kaspa_addresses::Prefix;
use kaspa_txscript::{extract_script_pub_key_address, pay_to_script_hash_script};
fn main() {
    let s: Vec<u8> = vec![8,100,0,0,0,0,0,0,0,118,82,121,147,118,0,162,105,185,203,81,156,105,118,88,205,1,8,124,126,185,118,201,118,1,68,148,89,147,124,188,126,170,2,0,0,1,170,126,1,32,126,124,126,1,135,126,185,0,204,195,135,105,0,122,117,117,117,81];
    let spk = pay_to_script_hash_script(&s);
    let addr = extract_script_pub_key_address(&spk, Prefix::Testnet).expect("addr");
    println!("V3 REP100 GENESIS ADDRESS:");
    println!("{}", addr);
}
