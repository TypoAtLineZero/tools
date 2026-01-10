use std::thread;
use rayon::prelude::*;

fn main() {
    let mut my_vec: Vec<i64> = Vec::new();

    thread::spawn(move || {
        add_to_vec(& mut my_vec);
    });

    my_vec.push(34);

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(256)
        .build()
        .unwrap();

    pool.install(|| {
        let scan_result: Vec<Subdomain> = subdomains::enumrate(&http_client, target)
            .unwrap()
            .into_par_iter()
            .map(ports::scan_ports)
            .collect();

        for subdomain in scan_result {
            println!("{}:", &subdomain.domain);
            for port in &subdomain.open_ports {
                println!("  {}", port.port);
            }

            println!("");
        }
    });
}

fn add_to_vec(vec: &mut Vec<i64>) {
    vec.push(42);
}
