#![cfg(feature = "full")]

use std::sync::Arc;
use tokio::sync::Semaphore;

#[test]
fn no_permits() {
    // this should not panic
    Semaphore::new(0);
}

#[test]
fn try_acquire() {
    let sem = Semaphore::new(1);
    {
        let p1 = sem.try_acquire();
        assert!(p1.is_ok());
        let p2 = sem.try_acquire();
        assert!(p2.is_err());
    }
    let p3 = sem.try_acquire();
    assert!(p3.is_ok());
}

#[test]
fn try_acquire_n() {
    let sem = Arc::new(Semaphore::new(3));
    {
        let p1 = sem.try_acquire_n(4);
        assert!(p1.is_err());
        let p2 = sem.try_acquire_n(2);
        match p2.as_ref() {
            Ok(p2) => assert_eq!(2, p2.num_permits()),
            Err(_) => panic!(),
        };
        let p3 = sem.try_acquire_n(2);
        assert!(p3.is_err());
    }
    let p4 = sem.try_acquire();
    assert!(p4.is_ok());
}

#[tokio::test]
async fn acquire() {
    let sem = Arc::new(Semaphore::new(1));
    let p1 = sem.try_acquire().unwrap();
    let sem_clone = sem.clone();
    let j = tokio::spawn(async move {
        let _p2 = sem_clone.acquire().await;
    });
    drop(p1);
    j.await.unwrap();
}

#[tokio::test]
async fn acquire_n() {
    let sem = Arc::new(Semaphore::new(5));
    let p1 = sem.try_acquire_n(3).unwrap();
    let sem_clone = sem.clone();
    let j = tokio::spawn(async move {
        let _p2 = sem_clone.acquire_n(2).await;
    });
    drop(p1);
    j.await.unwrap();
}

#[tokio::test]
async fn add_permits() {
    let sem = Arc::new(Semaphore::new(0));
    let sem_clone = sem.clone();
    let j = tokio::spawn(async move {
        let _p2 = sem_clone.acquire().await;
    });
    sem.add_permits(1);
    j.await.unwrap();
}

#[test]
fn forget() {
    let sem = Arc::new(Semaphore::new(1));
    {
        let p = sem.try_acquire().unwrap();
        assert_eq!(sem.available_permits(), 0);
        p.forget();
        assert_eq!(sem.available_permits(), 0);
    }
    assert_eq!(sem.available_permits(), 0);
    assert!(sem.try_acquire().is_err());
}

#[test]
fn release_n() {
    let sem = Arc::new(Semaphore::new(3));
    {
        let p1 = sem.try_acquire_n(3);
        assert!(p1.is_ok());
        let mut p1 = p1.unwrap();
        p1.release_n(2);
        let p2 = sem.try_acquire_n(2);
        assert!(p2.is_ok());
    }
    let p3 = sem.try_acquire_n(3);
    assert!(p3.is_ok());
}

#[test]
#[should_panic(expected = "No enough permits available")]
fn release_n_panic() {
    let sem = Arc::new(Semaphore::new(3));
    let p1 = sem.try_acquire_n(3);
    assert!(p1.is_ok());
    let mut p1 = p1.unwrap();
    p1.release_n(4);
}

#[test]
fn forget_n() {
    let sem = Arc::new(Semaphore::new(3));
    {
        let mut p = sem.try_acquire_n(3).unwrap();
        assert_eq!(sem.available_permits(), 0);
        p.forget_n(3);
        assert_eq!(sem.available_permits(), 0);
    }
    assert_eq!(sem.available_permits(), 0);
    assert!(sem.try_acquire().is_err());
}

#[test]
#[should_panic(expected = "No enough permits available")]
fn forget_n_panic() {
    let sem = Arc::new(Semaphore::new(3));
    let mut p = sem.try_acquire_n(3).unwrap();
    assert_eq!(sem.available_permits(), 0);
    p.forget_n(4);
}

#[tokio::test]
async fn stresstest() {
    let sem = Arc::new(Semaphore::new(5));
    let mut join_handles = Vec::new();
    for _ in 0..1000 {
        let sem_clone = sem.clone();
        join_handles.push(tokio::spawn(async move {
            let _p = sem_clone.acquire().await;
        }));
    }
    for j in join_handles {
        j.await.unwrap();
    }
    // there should be exactly 5 semaphores available now
    let _p1 = sem.try_acquire().unwrap();
    let _p2 = sem.try_acquire().unwrap();
    let _p3 = sem.try_acquire().unwrap();
    let _p4 = sem.try_acquire().unwrap();
    let _p5 = sem.try_acquire().unwrap();
    assert!(sem.try_acquire().is_err());
}
