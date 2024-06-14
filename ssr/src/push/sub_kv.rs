use std::sync::Arc;

use redb::{Database, ReadableTable, TableDefinition};
use sha2::{Digest, Sha512};
use tokio::task::spawn_blocking;
use web_push::SubscriptionInfo;

const TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("subscriptions");

#[derive(Clone)]
pub struct SubKV(Arc<Database>);

impl SubKV {
    pub fn new() -> Result<Self, redb::Error> {
        let db = Database::create("./redb-kv.db")?;
        let write_txn = db.begin_write()?;
        {
            write_txn.open_table(TABLE)?;
        }
        write_txn.commit()?;
        Ok(Self(Arc::new(db)))
    }

    fn spawn_blocking<F, R>(&self, f: F) -> tokio::task::JoinHandle<Result<R, redb::Error>>
    where
        F: FnOnce(&Database) -> Result<R, redb::Error> + Send + 'static,
        R: Send + 'static,
    {
        let db = self.0.clone();
        spawn_blocking(move || f(&db))
    }

    pub async fn add_subscription(&self, subscription: SubscriptionInfo) -> Result<(), redb::Error> {
        self.spawn_blocking(move |db| {
            let raw = serde_json::to_vec(&subscription).unwrap();
            let sub_hash = Sha512::digest(&raw);
            let write_txn = db.begin_write()?;
            {
                let mut table = write_txn.open_table(TABLE)?;
                table.insert(sub_hash.as_slice(), raw.as_slice())?;
            }
            Ok(())
        }).await.unwrap()
    }

    pub async fn all_subscriptions(&self) -> Result<Vec<SubscriptionInfo>, redb::Error> {
        self.spawn_blocking(move |db| {
            let read_txn = db.begin_read()?;
            let subs = {
                let table = read_txn.open_table(TABLE)?;
                table.iter()?
                    .map(|res| res.map(|(_, ag)| {
                        serde_json::from_slice(ag.value()).unwrap()
                    }))
                    .collect::<Result<_, _>>()?
            };
            Ok(subs)
        }).await.unwrap()
    }
}