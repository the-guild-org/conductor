use conductor::run_services;
use conductor_config::LoggerConfig;
use tracing::subscriber::set_global_default;
use tracing_subscriber::layer::SubscriberExt;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let default_logger_config = LoggerConfig::default();
  let global_logger = conductor_logger::logger_layer::build_logger(
    &default_logger_config.format,
    &default_logger_config.filter,
    default_logger_config.print_performance_info,
  )
  .expect("failed to build logger");
  set_global_default(tracing_subscriber::registry().with(global_logger))
    .expect("failed to set global default logger");

  let config_file_path = std::env::args()
    .nth(1)
    .unwrap_or("./config.json".to_string());

  run_services(&config_file_path).await
}
