use multidustry_transport::Reflectionable;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Foo {
    some: i32,
}

impl Reflectionable for Foo {
    fn stable_type_hash() -> &'static str {
        "Foo"
    }
}

#[derive(Serialize, Deserialize)]
struct Bar {
    some: i32,
}

impl Reflectionable for Bar {
    fn stable_type_hash() -> &'static str {
        "Bar"
    }
}

#[tokio::test]
async fn inproc_test() {
    let uuid = Uuid::new_v4();

    let handle = tokio::spawn(async move {
        let (tx, rx) = TransportBuilder::<Foo, Bar>::server(uuid.clone()).build();
        let res = rx.recv().await.unwrap();
        assert_eq!(res, Foo { some: 15 });
        let _ = tx.send().await;
    });

    let (tx, rx) = TransportBuilder::<Foo, Bar>::client(uuid.clone())
        .with_timeout(Duration::from_secs(5))
        .with_retry(3)
        .error_strategy(ErrorStrategy::Drop)
        .guarantees(Guarantees::Reliable)
        .build();
    let _ = tx.send(Foo { some: 15 }).await;
    let result = rx.recv().await.unwrap();
    assert_eq!(result, Bar { some: 18 });
    handle.await.unwrap();
}
