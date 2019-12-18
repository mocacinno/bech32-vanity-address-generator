use bitcoin::network::constants::Network;
use bitcoin::util::address::Address;
use bitcoin::util::key::PrivateKey;
use secp256k1::rand::thread_rng;
use secp256k1::Secp256k1;
use std::env;
use std::sync::{atomic::AtomicBool, atomic::AtomicU64, atomic::Ordering, Arc};
use std::time::SystemTime;

const CHARSET: [char; 32] = [
    'q', 'p', 'z', 'r', 'y', '9', 'x', '8', 'g', 'f', '2', 't', 'v', 'd', 'w', '0', 's', '3', 'j',
    'n', '5', '4', 'k', 'h', 'c', 'e', '6', 'm', 'u', 'a', '7', 'l',
];

fn run(id: i32, prefix: String, counter: Arc<AtomicU64>, flag: Arc<AtomicBool>) {
    let secp = Secp256k1::new();
    let mut local_count = 0;
    let start_time = SystemTime::now();
    let sync_num = 10000;
    let log_num = 100000;
    let estimated_hash_num = 32.0_f64.powi(prefix.len() as i32 - 4); // except 'bc1q'
    loop {
        let raw_priv_key = secp256k1::key::SecretKey::new(&mut thread_rng());
        let private_key = PrivateKey {
            compressed: true,
            network: Network::Bitcoin,
            key: raw_priv_key,
        };
        let address = Address::p2wpkh(&private_key.public_key(&secp), Network::Bitcoin);
        local_count += 1;

        if local_count % sync_num == 0 {
            if flag.load(Ordering::SeqCst) {
                break;
            }
            counter.fetch_add(sync_num, Ordering::SeqCst);
        }

        if id == 0 && local_count % log_num == 0 {
            let elapsed_secs = start_time.elapsed().unwrap().as_millis() as f64 / 1000.0;
            let total_count = counter.load(Ordering::SeqCst);
            let speed = (total_count as f64) / elapsed_secs;
            let time_left = (estimated_hash_num - total_count as f64) / speed;
            println!(
                "count: {}\telapsed: {:.2}min\tspeed: {:.2}/s\tprogress(est): {:.2}%\tleft(est): {:.2}min",
                total_count,
                elapsed_secs / 60.0,
                speed,
                ((total_count as f64) / estimated_hash_num * 100.0),
                time_left / 60.0
            );
        }

        if address.to_string().starts_with(&prefix) {
            println!("result:");
            println!("privkey:\t{}", private_key);
            println!("address:\t{}", address.to_string());
            flag.store(true, Ordering::SeqCst);
            break;
        }
    }
}

fn main() {
    if env::args().len() < 2 {
        eprintln!(
            "usage: {} <the address prefix to match>",
            env::args().nth(0).unwrap()
        );
        return;
    }
    let args: Vec<String> = env::args().collect();
    for c in args[1].chars() {
        if !CHARSET.contains(&c) {
            eprintln!("invalid char: {}", c);
            return;
        }
    }

    let prefix: String = "bc1q".to_string() + &args[1];
    println!("checking prefix {}", prefix);

    let counter = Arc::new(AtomicU64::new(0));
    let flag = Arc::new(AtomicBool::new(false));
    let thread_num = num_cpus::get() / 2;
    let mut threads = Vec::new();
    for idx in 0..thread_num {
        let local_counter = counter.clone();
        let local_flag = flag.clone();
        let prefix = prefix.clone();
        let thread = std::thread::spawn(move || run(idx as i32, prefix, local_counter, local_flag));
        threads.push(thread);
    }
    for thread in threads {
        thread.join().unwrap();
    }
}