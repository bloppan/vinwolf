use std::{thread::{self, sleep}, time::Duration, vec};
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

fn example_3() {

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

fn example_4() {

    let data = vec![1, 2, 3, 4, 5];
    
    let results = Arc::new(Mutex::new(Vec::new()));

    thread::scope(|s| {
        for num in &data {
            let ref_results = Arc::clone(&results);
            s.spawn(move || {
                let cuadrado = num * num;
                let mut ref_results = ref_results.lock().unwrap();
                ref_results.push(cuadrado);
            });
        }
    });

    let results = results.lock().unwrap();
    println!("results: {:?}", results);
}

fn main() {

    let data = vec![1, 2, 3, 4, 5];
    let mut vector: Vec<(u32, u32)> = Vec::new();
    vector.push((10, 20));

    let arc_vector = Arc::new(Mutex::new(&vector));
    let results = Arc::new(Mutex::new(Vec::new()));

    thread::scope(|s| {
        for num in &data {
            let ref_vector = Arc::clone(&arc_vector);
            let ref_results = Arc::clone(&results);
            s.spawn(move || {
                let cuadrado = num * num;

                let mut ref_results = ref_results.lock().unwrap();
                ref_results.push(cuadrado);
            });
        }
    });
    
    println!("vector: {:?}", vector);
    let results = results.lock().unwrap();
    println!("results: {:?}", results);
}


