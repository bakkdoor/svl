use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use tokio::task;

pub use cozo::{
    DataValue, Error as DBError, JsonData, MultiTransaction, NamedRows, Num, Validity, Vector,
};
use cozo::{DbInstance, ScriptMutability};

pub type DBResult = Result<NamedRows, DBError>;
pub type DBParams = BTreeMap<String, DataValue>;

pub struct DBConnection {
    db: Arc<Mutex<DbInstance>>,
}

impl DBConnection {
    pub async fn new() -> Result<Self, String> {
        let db = DbInstance::new_with_str("rocksdb", "svl-stats.db", Default::default())?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }

    pub async fn run_immutable(&self, script: &str, params: DBParams) -> DBResult {
        let db = Arc::clone(&self.db);
        let script = script.to_string();
        let params = params.clone();
        task::spawn_blocking(move || {
            let db = db.lock().unwrap();
            db.run_script(&script, params, ScriptMutability::Immutable)
        })
        .await
        .unwrap()
    }

    pub async fn run_mutable(&self, script: &str, params: DBParams) -> DBResult {
        let db = Arc::clone(&self.db);
        let script = script.to_string();
        let params = params.clone();
        task::spawn_blocking(move || {
            let db = db.lock().unwrap();
            db.run_script(&script, params, ScriptMutability::Mutable)
        })
        .await
        .unwrap()
    }

    pub fn multi_tx(&self, write: bool) -> AsyncMultiTransaction {
        let db = Arc::clone(&self.db);
        let db = db.lock().unwrap();
        let tx = db.multi_transaction(write);
        AsyncMultiTransaction(tx)
    }
}

pub struct AsyncMultiTransaction(MultiTransaction);

impl AsyncMultiTransaction {
    pub async fn commit(self) -> Result<(), DBError> {
        let tx = self.0;
        task::spawn_blocking(move || tx.commit()).await.unwrap()
    }

    pub fn run_script(&self, script: &str, params: DBParams) -> DBResult {
        let AsyncMultiTransaction(tx) = self;
        tx.run_script(script, params)
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
