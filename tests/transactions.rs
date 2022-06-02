use cephalopod::{
    engine,
    parse::{Tx, Type},
};
use tokio::sync::mpsc;

#[tokio::test]
async fn basics() {
    let txs: Vec<Tx> = vec![
        Tx {
            id: 9,
            typ: Type::Resolve,
            client: 6,
            amount: 0.,
        },
        Tx {
            id: 3,
            typ: Type::Deposit,
            client: 2,
            amount: 1.5,
        },
        Tx {
            id: 7,
            typ: Type::Deposit,
            client: 4,
            amount: 2.5,
        },
        Tx {
            id: 1,
            typ: Type::Deposit,
            client: 1,
            amount: 1.,
        },
        Tx {
            id: 2,
            typ: Type::Deposit,
            client: 2,
            amount: 1.5,
        },
        Tx {
            id: 5,
            typ: Type::Deposit,
            client: 3,
            amount: 2.0,
        },
        Tx {
            id: 4,
            typ: Type::Withdrawal,
            client: 2,
            amount: 0.5,
        },
        Tx {
            id: 3,
            typ: Type::Dispute,
            client: 2,
            amount: 0.,
        },
        Tx {
            id: 10,
            typ: Type::Dispute,
            client: 7,
            amount: 0.,
        },
        Tx {
            id: 5,
            typ: Type::Dispute,
            client: 3,
            amount: 0.,
        },
        Tx {
            id: 6,
            typ: Type::Deposit,
            client: 4,
            amount: 1.0,
        },
        Tx {
            id: 5,
            typ: Type::Resolve,
            client: 3,
            amount: 0.,
        },
        Tx {
            id: 6,
            typ: Type::Dispute,
            client: 4,
            amount: 0.,
        },
        Tx {
            id: 8,
            typ: Type::Chargeback,
            client: 5,
            amount: 0.,
        },
        Tx {
            id: 6,
            typ: Type::Chargeback,
            client: 4,
            amount: 0.,
        },
    ];

    let (tx, rx) = mpsc::channel(txs.len());
    let parser = tokio::spawn(async move {
        for elem in txs {
            tx.send(elem).await.unwrap();
        }
    });
    let (parsed, ledger) = tokio::join!(parser, engine::process(rx));
    assert!(parsed.is_ok());
    assert_eq!(ledger.len(), 4);

    assert!(ledger.contains_key(&1));
    let acct = ledger.get(&1).unwrap();
    assert_eq!(acct.available, 1.);
    assert_eq!(acct.held, 0.);
    assert_eq!(acct.locked, false);

    assert!(ledger.contains_key(&2));
    let acct = ledger.get(&2).unwrap();
    assert_eq!(acct.available, 1.);
    assert_eq!(acct.held, 1.5);
    assert_eq!(acct.locked, false);

    assert!(ledger.contains_key(&3));
    let acct = ledger.get(&3).unwrap();
    assert_eq!(acct.available, 2.);
    assert_eq!(acct.held, 0.);
    assert_eq!(acct.locked, false);

    assert!(ledger.contains_key(&4));
    let acct = ledger.get(&4).unwrap();
    assert_eq!(acct.available, 2.5);
    assert_eq!(acct.held, 0.);
    assert_eq!(acct.locked, true);

    assert!(!ledger.contains_key(&5));
    assert!(!ledger.contains_key(&6));
    assert!(!ledger.contains_key(&7));
}

#[tokio::test]
async fn frozen() {
    let txs: Vec<Tx> = vec![
        Tx {
            id: 1,
            typ: Type::Deposit,
            client: 1,
            amount: 2.0,
        },
        Tx {
            id: 2,
            typ: Type::Deposit,
            client: 1,
            amount: 1.0,
        },
        Tx {
            id: 2,
            typ: Type::Dispute,
            client: 1,
            amount: 0.,
        },
        Tx {
            id: 2,
            typ: Type::Chargeback,
            client: 1,
            amount: 0.,
        },
        Tx {
            id: 3,
            typ: Type::Deposit,
            client: 1,
            amount: 1.5,
        },
        Tx {
            id: 4,
            typ: Type::Withdrawal,
            client: 1,
            amount: 3.0,
        },
    ];

    let (tx, rx) = mpsc::channel(txs.len());
    let parser = tokio::spawn(async move {
        for elem in txs {
            tx.send(elem).await.unwrap();
        }
    });
    let (parsed, ledger) = tokio::join!(parser, engine::process(rx));
    assert!(parsed.is_ok());
    assert_eq!(ledger.len(), 1);

    assert!(ledger.contains_key(&1));
    let acct = ledger.get(&1).unwrap();
    assert_eq!(acct.available, 3.5);
    assert_eq!(acct.held, 0.);
    assert_eq!(acct.locked, true);
}

#[ignore]
#[tokio::test]
async fn limits() {
    let (tx, rx) = mpsc::channel(32);
    let mut transaction = Tx {
        id: 1,
        typ: Type::Deposit,
        client: 1,
        amount: 1.,
    };
    let max: u32 = 10000000;
    let parser = tokio::spawn(async move {
        for _ in 0..max {
            tx.send(transaction.clone()).await.unwrap();
            transaction.id += 1;
        }
    });

    let (parsed, ledger) = tokio::join!(parser, engine::process(rx));
    assert!(parsed.is_ok());
    assert_eq!(ledger.len(), 1);

    assert!(ledger.contains_key(&1));
    let acct = ledger.get(&1).unwrap();
    assert_eq!(acct.available, f64::from(max));
    assert_eq!(acct.held, 0.);
    assert_eq!(acct.locked, false);

    let (tx, rx) = mpsc::channel(32);
    let parser = tokio::spawn(async move {
        for i in 0..u16::MAX {
            tx.send(Tx {
                id: u32::from(i),
                typ: Type::Deposit,
                client: i,
                amount: 1.,
            })
            .await
            .unwrap();
        }
    });

    let (parsed, ledger) = tokio::join!(parser, engine::process(rx));
    assert!(parsed.is_ok());
    assert_eq!(ledger.len(), usize::from(u16::MAX));

    for i in 0..u16::MAX {
        assert!(ledger.contains_key(&i));
        let acct = ledger.get(&i).unwrap();
        assert_eq!(acct.available, 1.);
        assert_eq!(acct.held, 0.);
        assert_eq!(acct.locked, false);
    }
}
