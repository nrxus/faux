use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

#[faux::create]
pub struct Foo {}

#[faux::methods]
impl Foo {
    pub fn foo(&self) {
        todo!()
    }
    pub fn bar(&self) {
        todo!()
    }
}

#[test]
fn mock_multi_threaded_access() {
    let mut fake = Foo::faux();
    let done_count = Arc::new(AtomicUsize::new(0));

    faux::when!(fake.bar).then(move |()| {});

    let shared_fake1 = Arc::new(fake);
    let shared_fake2 = shared_fake1.clone();

    let dc1 = done_count.clone();
    let _t1 = std::thread::spawn(move || {
        for _ in 0..10000 {
            shared_fake1.bar();
        }
        dc1.fetch_add(1, Ordering::Relaxed);
    });

    let dc2 = done_count.clone();
    let _t2 = std::thread::spawn(move || {
        for _ in 0..10000 {
            shared_fake2.bar();
        }
        dc2.fetch_add(1, Ordering::Relaxed);
    });

    std::thread::sleep(Duration::from_millis(100)); // FIXME maybe we can do better?
    assert_eq!(done_count.load(Ordering::Relaxed), 2);
}

fn spin_until(a: &Arc<AtomicUsize>, val: usize) {
    loop {
        if a.load(Ordering::SeqCst) == val {
            break;
        }
    }
}

#[test]
fn mutex_does_not_lock_entire_mock() {
    // Assume calling a function lock the entire mock. Then a following scenario can happen:
    // * Thread 1 takes a lock L.
    // * Thread 2 calls mocked foo(), which tries to take and block on L.
    // * While holding L, thread 1 calls mocked bar(), blocking on the mock.
    // * We get a deadlock, even though bar() is seemingly unrelated to lock-taking foo().

    let mut fake = Foo::faux();
    let l = Arc::new(Mutex::new(10));
    let l_foo = l.clone();

    let done_count = Arc::new(AtomicUsize::new(0));
    let call_order = Arc::new(AtomicUsize::new(0));

    let co_foo = call_order.clone();
    faux::when!(fake.foo).then(move |()| {
        co_foo.swap(2, Ordering::SeqCst); // Let thread 1 call bar()
        let _ = l_foo.lock();
        spin_until(&co_foo, 3); // Hold the lock until thread 1 returns from bar()
    });
    faux::when!(fake.bar).then(move |()| {});

    let shared_fake1 = Arc::new(fake);
    let shared_fake2 = shared_fake1.clone();

    let dc1 = done_count.clone();
    let co1 = call_order.clone();
    let _t1 = std::thread::spawn(move || {
        let _ = l.lock();
        co1.swap(1, Ordering::SeqCst);
        spin_until(&co1, 2); // Wait for thread 2 to call foo
        shared_fake1.bar();
        co1.swap(3, Ordering::SeqCst);
        dc1.fetch_add(1, Ordering::Relaxed);
    });

    let dc2 = done_count.clone();
    let co2 = call_order.clone();
    let _t2 = std::thread::spawn(move || {
        spin_until(&co2, 1); // Wait for thread 1 to grab L
        shared_fake2.foo();
        dc2.fetch_add(1, Ordering::Relaxed);
    });

    std::thread::sleep(Duration::from_millis(100)); // FIXME maybe we can do better?
    assert_eq!(done_count.load(Ordering::Relaxed), 2);
}
