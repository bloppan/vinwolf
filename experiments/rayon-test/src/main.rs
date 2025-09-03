use core::num;
use std::collections::VecDeque;
use std::hash::Hash;
use std::sync::Arc;

use rayon::prelude::*;
use rayon::vec;
use rayon::ThreadPoolBuilder;
use std::collections::HashMap;

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

fn fold_reduce_example_2() {
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

fn suma(vector: &[u8]) -> u32 {
    vector.iter().fold(0, |acc, &x| acc + x as u32)
}

fn fold_reduce_example_3() {

    let mut items: Vec<(u32, Vec<u8>)> = vec![];
    items.push((1, vec![1,2,3,4]));
    items.push((2, vec![2,4,8,10]));
    items.push((3, vec![3,4,5,6,10]));

    let result: Vec<(u32, u32)> = items
            .par_iter()
            .fold(
                || Vec::<(u32, u32)>::new(),
                |mut acc, orig| {
                    let sum = orig.1.iter().fold(0, |total, res| total + *res as u32);
                    acc.push((orig.0, sum));
                    acc
                },
            )
            .reduce(
                || Vec::<(u32, u32)>::new(),
                |mut acc, vec_sum| {
                    acc.extend(vec_sum);
                    acc
                }
            );

    for entry in &result {
        println!("key-value: {:?}", entry);
    }
}


fn try_fold_try_reduce_example() {

    let nums = vec![1_i32, 2, 30, 4, 5];

    let result: Result<i32, &'static str> = nums
        .par_iter()
        .try_fold(
            || 0,
            |acc: i32, &x: &i32| {
                if x > 5 { Err("Número demasiado grande") } else { println!("bien"); Ok(acc + x) }
            },
        )
        .try_reduce(
            || 0,
            |a: i32, b: i32| Ok(a + b),
        );

    println!("{result:?}");
}

fn accumulate_vector(vector: &[u8]) -> Result<u32, &'static str> {

    if vector.is_empty() {
        return Err("Empty vector");
    }

    //println!("Ok");
    Ok(vector.iter().fold(0, |acc, value| acc + *value as u32))
}

fn try_fold_try_reduce_example_2() {

    let mut items: Vec<(u32, Vec<u8>)> = vec![];
    items.push((1, vec![1,2,3,4]));
    items.push((2, vec![2,4,8,10]));
    items.push((3, vec![3,4,5,6,10]));
    items.push((4, vec![3,4,5,6,10]));
    items.push((5, vec![3,4,5,6,10]));
    //items.push((6, vec![]));
    items.push((6, vec![3,4,5,6,10]));
    items.push((7, vec![3,4,5,6,10]));
    items.push((8, vec![3,4,5,6,10]));
    items.push((9, vec![3,4,5,6,10]));
    items.push((10, vec![3,4,5,6,10]));
    items.push((11, vec![3,4,5,6,10]));
    items.push((12, vec![3,4,5,6,10]));
    items.push((13, vec![3,4,5,6,10]));
    items.push((14, vec![3,4,5,6,10]));
    items.push((15, vec![3,4,5,6,10]));
    items.push((16, vec![3,4,5,6,10]));
    items.push((17, vec![3,4,5,6,10]));
    items.push((18, vec![3,4,5,6,10]));
    items.push((19, vec![3,4,5,6,10]));
    items.push((20, vec![3,4,5,6,10]));
    items.push((21, vec![3,4,5,6,10]));
    items.push((22, vec![3,4,5,6,10]));
    items.push((23, vec![3,4,5,6,10]));
    items.push((24, vec![3,4,5,6,10]));
    items.push((25, vec![3,4,5,6,10]));
    items.push((26, vec![3,4,5,6,10]));
    items.push((27, vec![3,4,5,6,10]));
    items.push((28, vec![3,4,5,6,10]));
    items.push((29, vec![3,4,5,6,10]));
    items.push((30, vec![3,4,5,6,10]));
    items.push((31, vec![3,4,5,6,10]));
    items.push((32, vec![3,4,5,6,10]));
    items.push((33, vec![3,4,5,6,10]));
    items.push((34, vec![3,4,5,6,10]));
    items.push((35, vec![3,4,5,6,10]));
    items.push((36, vec![3,4,5,6,10]));
    items.push((37, vec![3,4,5,6,10]));
    items.push((38, vec![3,4,5,6,10]));
    items.push((39, vec![3,4,5,6,10]));
    items.push((40, vec![3,4,5,6,10]));
    items.push((41, vec![3,4,5,6,10]));
    items.push((42, vec![3,4,5,6,10]));

    let map: Result<Vec<(u32, u32)>, &'static str> = items
                        .par_iter()
                        .try_fold(
                            || Vec::new(),
                            |mut acc: Vec<(u32, u32)>, value| {
                                
                                match accumulate_vector(&value.1) {
                                    Ok(acc_result) => {
                                        acc.push((value.0, acc_result));
                                        Ok(acc)
                                    },
                                    Err(e) => Err(e),
                                }
                            },
                        )
                        .try_reduce(
                            || Vec::new(),
                            | mut acc: Vec<(u32, u32)>, single_vector | { 
                                
                                acc.extend(single_vector);
                                Ok(acc)
                            
                            }
                        );

    for item in &map.unwrap() {
        println!("{:?}", item)
    }
    //println!("result: {:?}", map);
}

fn main() {

    let mut vector: VecDeque<u32> = VecDeque::new();
    vector.push_back(1);
    vector.push_back(2);
    vector.push_back(3);
    vector.pop_front();
    
    println!("vector: {:?}", vector);
}




