use tracing::subscriber::set_global_default;
use tracing_subscriber::filter::EnvFilter;

pub fn init_log() {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(false)
        .with_line_number(true)
        .with_thread_ids(false)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    set_global_default(subscriber).expect("set global subscriber fail");
}
