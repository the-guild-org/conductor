use conductor::run_services;
use num_cpus;
use tokio::runtime;

fn main() {
    let config_file_path = std::env::args()
        .nth(1)
        .unwrap_or("./config.json".to_string());

    let cpu_count = num_cpus::get();
    let runtime = runtime::Builder::new_multi_thread()
        .worker_threads(cpu_count)
        .max_blocking_threads(cpu_count * 4)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(run_services(config_file_path));
}
