use crate::connection::Connection;
use crate::error::Result;
use bytes::Bytes;
use std::sync::Arc;

pub struct Client {
    connection: Connection,
}

impl Client {
    pub async fn connect(
        addr: &str,
        user: &str,
        password: Option<&str>,
        database: Option<&str>,
    ) -> Result<Self> {
        let connection = Connection::connect(addr, user, password, database).await?;
        Ok(Self { connection })
    }

    pub async fn execute(&mut self, query: &str) -> Result<()> {
        self.connection.execute(query).await
    }

    pub async fn query(&mut self, query: &str) -> Result<Vec<Row>> {
        self.connection.query(query).await
    }
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub table_oid: u32,
    pub column_id: i16,
    pub type_oid: u32,
    pub type_len: i16,
    pub type_mod: i32,
    pub format: i16,
}

use postgres_types::{FromSql, Type};

pub struct Row {
    pub columns: Arc<Vec<Column>>,
    pub data: Vec<Option<Bytes>>,
}

impl Row {
    pub fn get<'a, T: FromSql<'a>>(&'a self, index: usize) -> Result<T> {
        let col = self.columns.get(index).ok_or_else(|| {
            crate::error::Error::Parse(format!("Column index {} out of bounds", index))
        })?;
        let bytes = self.get_raw(index);

        let ty = Type::from_oid(col.type_oid).unwrap_or(Type::UNKNOWN);

        match bytes {
            Some(b) => T::from_sql(&ty, b).map_err(|e| crate::error::Error::Parse(e.to_string())),
            None => T::from_sql_null(&ty).map_err(|e| crate::error::Error::Parse(e.to_string())),
        }
    }

    pub fn get_raw(&self, index: usize) -> Option<&Bytes> {
        self.data.get(index).and_then(|opt| opt.as_ref())
    }
}
