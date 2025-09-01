use tokio::time::{sleep, Duration};

async fn example_1() {

    let task1 = tokio::spawn(async {
        println!("Tarea 1 iniciada");
        sleep(Duration::from_millis(1000)).await;
        println!("Tarea 1 terminada");
        "Resultado 1"
    });

    let task2 = tokio::spawn(async {
        println!("Tarea 2 iniciada");
        sleep(Duration::from_millis(500)).await;
        println!("Tarea 2 terminada");
        2
    });

    let (res1, res2) = tokio::join!(task1, task2);
    println!("Resultados: {:?}, {:?}", res1.unwrap(), res2.unwrap());
}


#[tokio::main]
async fn main() {
    
    example_1().await;
    println!("despues de example");
}