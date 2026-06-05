use kaspa_addresses::Prefix;
use kaspa_txscript::{extract_script_pub_key_address, pay_to_script_hash_script};
fn main() {
    let s: Vec<u8> = vec![8,105,0,0,0,0,0,0,0,120,147,118,0,162,105,0,195,1,8,82,121,88,205,126,185,191,130,89,124,127,126,170,2,0,0,1,170,126,1,32,126,124,126,1,135,126,135,105,0,122,117,117,81];
    let spk = pay_to_script_hash_script(&s);
    let addr = extract_script_pub_key_address(&spk, Prefix::Testnet).expect("addr");
    println!("V3 REP105 TARGET ADDRESS:");
    println!("{}", addr);
}
