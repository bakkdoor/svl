use std::collections::{BTreeMap, BTreeSet};

pub use cozo::{DataValue, Error as DBError, JsonData, NamedRows, Num, Validity, Vector};
use cozo::{DbInstance, ScriptMutability};

pub type DBResult = Result<NamedRows, DBError>;
pub type DBParams = BTreeMap<String, DataValue>;

pub struct DBConnection {
    db: DbInstance,
}

impl DBConnection {
    pub fn new() -> Result<Self, String> {
        let db = DbInstance::new_with_str("rocksdb", "svl-stats.db", Default::default())?;
        Ok(Self { db })
    }

    pub fn run_immutable(&self, script: &str, params: DBParams) -> DBResult {
        self.db
            .run_script(script, params, ScriptMutability::Immutable)
    }

    pub fn run_mutable(&self, script: &str, params: DBParams) -> DBResult {
        self.db
            .run_script(script, params, ScriptMutability::Mutable)
    }

    pub fn multi_tx(&self, write: bool) -> cozo::MultiTransaction {
        self.db.multi_transaction(write)
    }
}

pub trait ToDataValue {
    fn to_data_value(&self) -> DataValue;
}

impl<T> ToDataValue for &T
where
    T: ToDataValue,
{
    fn to_data_value(&self) -> DataValue {
        (**self).to_data_value()
    }
}

impl ToDataValue for i64 {
    fn to_data_value(&self) -> DataValue {
        DataValue::Num(Num::Int(*self))
    }
}

impl ToDataValue for f64 {
    fn to_data_value(&self) -> DataValue {
        DataValue::Num(Num::Float(*self))
    }
}

impl ToDataValue for bool {
    fn to_data_value(&self) -> DataValue {
        DataValue::Bool(*self)
    }
}

impl ToDataValue for String {
    fn to_data_value(&self) -> DataValue {
        DataValue::Str(self.into())
    }
}

impl ToDataValue for &str {
    fn to_data_value(&self) -> DataValue {
        DataValue::Str(self.to_string().into())
    }
}

impl ToDataValue for Vec<DataValue> {
    fn to_data_value(&self) -> DataValue {
        DataValue::List(self.clone())
    }
}

impl ToDataValue for BTreeSet<DataValue> {
    fn to_data_value(&self) -> DataValue {
        DataValue::Set(self.clone())
    }
}

impl ToDataValue for Vector {
    fn to_data_value(&self) -> DataValue {
        DataValue::Vec(self.clone())
    }
}

impl ToDataValue for Vec<u8> {
    fn to_data_value(&self) -> DataValue {
        DataValue::Bytes(self.clone())
    }
}

impl ToDataValue for JsonData {
    fn to_data_value(&self) -> DataValue {
        DataValue::Json(self.clone())
    }
}

impl ToDataValue for Validity {
    fn to_data_value(&self) -> DataValue {
        DataValue::Validity(*self)
    }
}

impl ToDataValue for DataValue {
    fn to_data_value(&self) -> DataValue {
        self.clone()
    }
}

impl ToDataValue for () {
    fn to_data_value(&self) -> DataValue {
        DataValue::Null
    }
}

impl ToDataValue for usize {
    fn to_data_value(&self) -> DataValue {
        DataValue::Num(Num::Int(*self as i64))
    }
}

pub fn val<V: ToDataValue>(v: V) -> DataValue {
    v.to_data_value()
}
