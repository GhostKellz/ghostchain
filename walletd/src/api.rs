// WalletD API Server (stub)
use anyhow::Result;

pub struct ApiServer;

impl ApiServer {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn run(&self) -> Result<()> {
        Ok(())
    }
}