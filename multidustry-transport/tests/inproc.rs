use multidustry_transport::{Reflectionable, transport_builder::TransportBuilder};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Foo {
    some: i32,
}

impl Reflectionable for Foo {
    fn stable_type_hash() -> &'static str {
        "Foo"
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
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
        let listener = TransportBuilder::<Foo, Bar>::server(uuid).build().await;
        let connection = listener.accept().await.unwrap();
        let (tx, rx) = connection.split();

        let req = rx.recv().await.unwrap();
        assert_eq!(req, Foo { some: 15 });

        tx.send(Bar { some: 18 }).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let connection = TransportBuilder::<Foo, Bar>::client(uuid)
        // .with_timeout(Duration::from_secs(5))
        // .with_retry(3)
        // .error_strategy(ErrorStrategy::Drop)
        // .guarantees(Guarantees::Reliable)
        .build()
        .await
        .unwrap();

    let (tx, rx) = connection.split();

    tx.send(Foo { some: 15 }).await.unwrap();
    let result = rx.recv().await.unwrap();
    assert_eq!(result, Bar { some: 18 });

    handle.await.unwrap();
}
