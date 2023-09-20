use std::collections::{BTreeMap, BTreeSet};

pub use cozo::{DataValue, JsonData, Num, Validity, Vector};
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

pub fn int_val(i: i64) -> DataValue {
    DataValue::Num(Num::Int(i))
}

pub fn float_val(f: f64) -> DataValue {
    DataValue::Num(Num::Float(f))
}

pub fn bool_val(b: bool) -> DataValue {
    DataValue::Bool(b)
}

pub fn string_val(s: &str) -> DataValue {
    DataValue::Str(s.into())
}

pub fn list_val(l: Vec<DataValue>) -> DataValue {
    DataValue::List(l)
}

pub fn set_val(s: BTreeSet<DataValue>) -> DataValue {
    DataValue::Set(s)
}

pub fn vec_val(v: Vector) -> DataValue {
    DataValue::Vec(v)
}

pub fn bytes_val(b: Vec<u8>) -> DataValue {
    DataValue::Bytes(b)
}

pub fn json_val(j: JsonData) -> DataValue {
    DataValue::Json(j)
}

pub fn validity_val(v: Validity) -> DataValue {
    DataValue::Validity(v)
}
