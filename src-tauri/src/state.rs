use std::sync::Arc;
use tokio::sync::Mutex;
use crate::rustustc::cas::client::CASClient;
use crate::rustustc::young::YouthService;

pub struct AppState {
  pub cas_client: Mutex<Option<Arc<CASClient>>>,
  pub youth_service: Mutex<Option<Arc<YouthService>>>,
}

impl AppState {
  pub fn new() -> Self {
    Self {
      cas_client: Mutex::new(None),
      youth_service: Mutex::new(None),
    }
  }
}