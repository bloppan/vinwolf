use tokio::time::{sleep, Duration};
use tokio::sync::mpsc::{channel, Sender, Receiver};
use tokio::sync::oneshot;

async fn sleep_ms(ms: u64) {
    sleep(Duration::from_millis(ms)).await;
}

#[derive(Debug)]
enum Message {
    Hello,
    World,
}

#[derive(Debug)]
enum Ctrl {
    Quit,   
    Health(oneshot::Sender<HealthResponse>),
}

#[derive(Debug)]
enum HealthResponse {
    Healthy,
    UnHealthy,
}


async fn message_generator(mut cc: Receiver<Ctrl>, channel: Sender<Message>) {
    loop {
        tokio::select! {
            res = channel.send(Message::Hello) => {
                match res {
                    Ok(()) => sleep_ms(100).await,
                    Err(_) => {
                        eprintln!("Error sending message");
                        break;
                    }
                }
            },
            ctl = cc.recv() => {
                println!("Received something...");
                match ctl {
                    Some(Ctrl::Quit) => { println!("Received Quit... Stopping"); break; },
                    None => break,
                    Some(Ctrl::Health(rtx)) => {
                        rtx.send(HealthResponse::Healthy).unwrap()
                    }
                }
            }
        }
    }
}


async fn file_sink(mut channel: Receiver<Message>) {

    while let Some(msg) = channel.recv().await {
        println!("msg: {:?}", msg);
    }
}

async fn example_02() -> Result<(), tokio::sync::mpsc::error::SendError<Ctrl>> {

    let (tx, rx) = channel::<Message>(10);
    let (ctx, crx) = channel::<Ctrl>(10);

    tokio::spawn(message_generator(crx, tx));
    tokio::spawn(file_sink(rx));
    sleep_ms(2000).await;

    println!("Health message sent...");
    let (rtx, rrx) = oneshot::channel();
    ctx.send(Ctrl::Health(rtx)).await?;
    let response = rrx.await;
    println!("Received health response!");


    println!("Quit message sent...");
    ctx.send(Ctrl::Quit).await?;
    println!("After send Quit");
    sleep_ms(2000).await;

    Ok(())
}

async fn msg_gen(tx: Sender<Message>, rx: Receiver<Ctrl>) {

    println!("Send Hello..");
    tx.send(Message::Hello).await.unwrap();
}

async fn msg_recv(mut rx: Receiver<Message>) {

    let response = rx.recv().await.unwrap();
    println!("Response: {:?}", response);
}

async fn example_01() -> Result<(), tokio::sync::mpsc::error::SendError<Ctrl>> {

    let (mtx, mrx) = channel::<Message>(10);
    let (ctx, crx) = channel::<Ctrl>(10);

    tokio::spawn(msg_gen(mtx, crx));
    println!("Sleep 2s");
    sleep_ms(2000).await;
    tokio::spawn(msg_recv(mrx));
    sleep_ms(1000).await;
    Ok(())
}

async fn do_work(name: &str, ms: u64) {
    println!("Starting {name}");
    sleep(Duration::from_millis(ms)).await;
    println!("Finished {name}");
}

async fn example_03() {
    tokio::join!(
        do_work("A", 500),
        do_work("B", 300),
        do_work("C", 1000),
    );
}

async fn example_04() {

    let (tx, mut rx) = channel(5);

    tokio::spawn(async move {
        for i in 0..5 {
            println!("Sending");
            tx.send(format!("msg {i}")).await.unwrap();
            sleep(Duration::from_millis(200)).await;
        }
    });

    while let Some(msg) = rx.recv().await {
        println!("Received: {msg}");
    }
}

async fn example_05() {

    let (tx, mut rx) = channel(10);
    tokio::spawn(async move {
        sleep(Duration::from_millis(1500)).await;
        tx.send("message").await.unwrap();
    });

    tokio::select! {
        Some(msg) = rx.recv() => println!("Got: {msg}"),
        _ = sleep(Duration::from_secs(1)) => println!("Timeout!"),
    }
}

async fn worker(mut ctrl: Receiver<Ctrl>, mut rx: Receiver<Message>) {
    loop {
        tokio::select! {
            Some(msg) = rx.recv() => println!("Received: {:?}", msg),
            Some(cmd) = ctrl.recv() => match cmd {
                Ctrl::Quit => { println!("Received Quit. Stopping."); break; }
                Ctrl::Health(resp) => { let _ = resp.send(HealthResponse::Healthy); }
            },
            else => break,
        }
    }
}

#[tokio::main]
async fn main() {
    let (msg_tx, msg_rx) = channel(10);
    let (ctrl_tx, ctrl_rx) = channel(2);

    tokio::spawn(worker(ctrl_rx, msg_rx));

    msg_tx.send(Message::Hello).await.unwrap();

    let (resp_tx, resp_rx) = oneshot::channel();
    ctrl_tx.send(Ctrl::Health(resp_tx)).await.unwrap();
    println!("Health check: {:?}", resp_rx.await.unwrap());

    ctrl_tx.send(Ctrl::Quit).await.unwrap();
}