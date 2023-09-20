use std::collections::BTreeMap;

pub use cozo::{DataValue, Num};
use cozo::{DbInstance, ScriptMutability};

pub struct DBConnection {
    db: DbInstance,
}

pub type DBParams = BTreeMap<String, DataValue>;

impl DBConnection {
    pub fn new() -> Result<Self, String> {
        let db = DbInstance::new_with_str("rocksdb", "svl-stats.db", Default::default())?;
        Ok(Self { db })
    }

    pub fn run_immutable(
        &self,
        script: &str,
        params: DBParams,
    ) -> Result<cozo::NamedRows, cozo::Error> {
        self.db
            .run_script(script, params, ScriptMutability::Immutable)
    }

    pub fn run_mutable(
        &self,
        script: &str,
        params: DBParams,
    ) -> Result<cozo::NamedRows, cozo::Error> {
        self.db
            .run_script(script, params, ScriptMutability::Mutable)
    }

    pub fn multi_tx(&self, write: bool) -> cozo::MultiTransaction {
        self.db.multi_transaction(write)
    }
}