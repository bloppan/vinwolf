use core::num;
use std::hash::Hash;

use rayon::prelude::*;
use rayon::ThreadPoolBuilder;


fn paralell_sum() {

    let start = std::time::Instant::now();
    let numbers: Vec<i32> = (0..10000).collect();
    let sum: i32 = numbers.iter().sum();
    let end = start.elapsed();
    println!("time normal sum: {:?}", end); // 

    let start = std::time::Instant::now();
    let numbers: Vec<i32> = (0..10000).collect();
    let sum: i32 = numbers.par_iter().sum();
    let end = start.elapsed();
    println!("time parallel sum: {:?}", end); // 

    println!("Suma paralela: {}", sum); // Debería ser 4950 (suma de 0 a 99)
}

fn duplicate_num() {

    let numbers: Vec<i32> = (0..10000).collect();
    let doubled: Vec<i32> = numbers.par_iter().map(|&x| x * 2).collect();
}


fn fold_reduce() {
    
    let start = std::time::Instant::now();
    let numbers: Vec<i32> = (0..10000).collect();
    let sum_per_thread: Vec<i32> = numbers
        .par_iter()
        .fold(|| Vec::new(), |mut acc, &x| {
                if x % 2 == 0 {
                    acc.push(x);
                }
                acc
            },
        )
        .reduce(
            || Vec::new(),  // Inicializa el acumulador final
            |mut a, b| {
                a.extend(b);
                a
            },
        );
    let end = start.elapsed();
    println!("time fold_reduce: {:?}", end); 
    println!("Pares acumulados: {:?}", sum_per_thread);

}

fn thread_pool_builder() {
    let pool = ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();

    let start = std::time::Instant::now();
    pool.install(|| {
        let numbers: Vec<i32> = (0..10000).collect();
        let sum: i32 = numbers.par_iter().sum();
        println!("Suma desde pool personalizado: {}", sum); // 4950
    });
    let end = start.elapsed();
    println!("time 1: {:?}", end); // 54uS

    println!("Pool terminado, usando hilos del sistema ahora");
    let start = std::time::Instant::now();
    let sum_default = (0..10000).into_par_iter().sum::<i32>();
    let end = start.elapsed();
    println!("time 2: {:?}", end); // 425uS
    println!("Suma con pool predeterminado: {}", sum_default); // 4950
}

fn fold_reduce_example_1() {

    let numbers: Vec<i32> = (0..100).collect();
    let total: i32 = numbers
        .par_iter()
        .fold(
            || 0,           // Acumulador inicial por hilo
            |acc, &x| acc + x,  // Acumulación local
        )
        .reduce(
            || 0,           // Valor inicial para la reducción (opcional si no hay hilos)
            |a, b| a + b,   // Combinar resultados de hilos
        );

    println!("Suma total: {}", total); // Debería ser 4950 (suma de 0 a 99)
}

fn suma(vector: &[u8]) -> u32 {
    vector.iter().fold(0, |acc, &x| acc + x as u32)
}


use std::collections::HashMap;

fn main() {

    let mut items: Vec<(u32, Vec<u8>)> = vec![];
    items.push((1, vec![1,2,3,4]));
    items.push((2, vec![2,4,8,10]));
    items.push((3, vec![3,4,5,6,10]));

    let map: HashMap<u32, u32> = items
                .par_iter()
                .fold(
                    || HashMap::new(),
                    |mut acc, &(id, ref vector)| {
                        acc.insert(id, suma(&vector));
                        acc
                    },
                )
                .reduce(
                    || HashMap::new(),
                    |mut map, b| {
                        map.extend(b);
                        map
                    }
                );
                
    for entry in &map {
        println!("key: {:?} value: {:?}", entry.0, entry.1);
    }

}



