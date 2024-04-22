use std::sync::OnceLock;

use dapr::{client::TonicClient, Client};

use crate::daprs::dapr_url_grpc;

static mut DAPR_CLIENT: OnceLock<Client<TonicClient>> = OnceLock::new();
static DAPR_INITIALIZED: OnceLock<tokio::sync::Mutex<bool>> = OnceLock::new();

pub async fn get_dapr_client() -> Result<&'static mut Client<TonicClient>, Box<dyn std::error::Error + Sync + Send>> {
    unsafe {
        let client_option = DAPR_CLIENT.get_mut();
        if let Some(client) = client_option {
            return Ok(client);
        }
        let initializing_mutex = DAPR_INITIALIZED.get_or_init(|| tokio::sync::Mutex::new(false));
        let initialized = initializing_mutex.lock().await;
        if !*initialized {
            let address = dapr_url_grpc().unwrap();
            let client = Client::<TonicClient>::connect(address).await?;
            let _ = DAPR_CLIENT.set(client);
        }
        drop(initialized);
        Ok(DAPR_CLIENT.get_mut().unwrap())
    }
}
