use std::{thread::{self, sleep}, time::Duration};
use std::sync::{mpsc, Arc, Mutex};

fn example_1() -> Result<i32, &'static str> {

    let handle: thread::JoinHandle<Result<i32, &'static str>> = thread::spawn(|| {
        println!("Launched thread");
        thread::sleep(Duration::from_secs(3));
        println!("Thread finished");
        Err("thread error")
    });

    println!("Waiting to finish thread");

    match handle.join() {
        Ok(Ok(value)) => { println!("Thread finished successfully"); Ok(value) },
        Ok(Err(e)) => Err(e),
        Err(_) => { eprintln!("the thread panics"); Err("panic")},  
    }
}

fn example_2() {

    let (tx, rx) = mpsc::channel();
    let tx2 = tx.clone();
    
    let handle = thread::spawn(move || {

        for i in 0..5 {
            tx.send(format!("msg {}", i)).unwrap();
            sleep(Duration::from_millis(500));
        }
    }); 

    let h2 = thread::spawn(move || {
        
        for i in 0..5 {
            tx2.send(format!("tx2 msg: {}", i)).unwrap();
            sleep(Duration::from_secs(2));
        }
    });

    for msg in rx {
        println!("msg received: {:?}", msg);
    }

    handle.join().unwrap();
    h2.join().unwrap();

}


fn main() {

    let counter = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();
    *counter.lock().unwrap() += 1;

    for i in 0..4 {
        let c = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for j in 0..1000 {
                *c.lock().unwrap() += 1;
            }
        }))
    }

    for h in handles {
        h.join().unwrap();
    }
    
    println!("counter: {:?}", counter.lock().unwrap());
}


