use std::cell::RefCell;
use std::collections::VecDeque;
use crate::client::Client;
use crate::error::Result;

thread_local! {
    static POOL: RefCell<VecDeque<Client>> = RefCell::new(VecDeque::new());
}

pub struct Pool {
    addr: String,
    user: String,
    password: Option<String>,
    database: Option<String>,
}

impl Pool {
    pub fn new(addr: &str, user: &str, password: Option<&str>, database: Option<&str>) -> Self {
        Self {
            addr: addr.to_string(),
            user: user.to_string(),
            password: password.map(|s| s.to_string()),
            database: database.map(|s| s.to_string()),
        }
    }

    pub async fn get(&self) -> Result<Client> {
        if let Some(client) = POOL.with(|p| p.borrow_mut().pop_front()) {
            return Ok(client);
        }
        Client::connect(&self.addr, &self.user, self.password.as_deref(), self.database.as_deref()).await
    }

    pub fn put(&self, client: Client) {
        POOL.with(|p| p.borrow_mut().push_back(client));
    }
}
