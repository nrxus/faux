use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc,
    },
    thread,
    time::Duration,
};

#[faux::create]
pub struct Foo {}

#[faux::methods]
impl Foo {
    pub fn foo(&self) {
        unreachable!()
    }

    pub fn bar(&self) {
        unreachable!()
    }
}

#[test]
fn mock_multi_threaded_access() {
    let mut fake = Foo::faux();
    faux::when!(fake.bar).then(move |_| {});

    let fake = Arc::new(fake);

    // calls for Foo::bar() 10K times in a row
    // and then increments the counter
    let start_thread = || {
        let fake = fake.clone();

        std::thread::spawn(move || {
            for _ in 0..10_000 {
                fake.bar();
            }
        })
    };

    let thread_1 = start_thread();
    let thread_2 = start_thread();

    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        thread_1.join().unwrap();
        thread_2.join().unwrap();
        sender.send(()).unwrap();
    });

    receiver.recv_timeout(Duration::from_millis(100)).unwrap();
}

#[test]
fn mutex_does_not_lock_entire_mock() {
    // calls for fake.foo() and fake.bar() in two separate threads
    // these calls are synchronized so neither can finish without the other starting
    // this asserts that the the mock store is NOT locked for the entire invocation

    let mut fake = Foo::faux();

    // holds the following states:
    // 0: neither mocked method has started
    // 1: fake.bar() has started -> fake.foo() will not finish until it is set
    // 2: fake.foo() has finished -> fake.bar() will not finish until it is set
    let call_synchronizer = Arc::new(AtomicUsize::new(0));

    let foo_synchronizer = call_synchronizer.clone();
    faux::when!(fake.foo).then(move |_| {
        // wait until `fake.bar()` has started
        spin_until(&foo_synchronizer, 1);

        // allow `fake.bar()` to finish
        foo_synchronizer.swap(2, Ordering::SeqCst);
    });

    let bar_synchronizer = call_synchronizer;
    faux::when!(fake.bar).then(move |_| {
        // let `fake.foo()` start
        // nothing should be blocking `fake.foo()` from starting
        // unless we are locking the entire mock store
        bar_synchronizer.swap(1, Ordering::SeqCst);

        // wait until `fake.foo()` has finished
        spin_until(&bar_synchronizer, 2);
    });

    let fake = Arc::new(fake);

    let bar_thread = {
        let fake = fake.clone();
        std::thread::spawn(move || fake.bar())
    };

    let foo_thread = {
        let fake = fake;
        std::thread::spawn(move || fake.foo())
    };

    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        foo_thread.join().unwrap();
        bar_thread.join().unwrap();
        sender.send(()).unwrap();
    });

    receiver
        .recv_timeout(Duration::from_millis(100))
        .expect("a deadlock occurred!");
}

fn spin_until(a: &Arc<AtomicUsize>, val: usize) {
    loop {
        if a.load(Ordering::SeqCst) == val {
            break;
        }
    }
}
