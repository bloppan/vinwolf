use std::time::{Duration, SystemTime};
use tools::log;

pub fn schedule_task_at<F>(when: SystemTime, task: F) 
where
        F: FnOnce() + Send + 'static,
{

    let now = SystemTime::now();
    //let target_time = UNIX_EPOCH + Duration::from_secs(slot as u64 * SLOT_PERIOD as u64);
    let target_time = when;

    let delay = match target_time.duration_since(now) {
        Ok(d) => d,
        Err(_) => Duration::from_secs(0),
    };

    std::thread::scope(move |_| {
        std::thread::sleep(delay);
        task();
    });
}

fn print_hello_task() {
    log::debug!("Hello task!");
}



#[test]
fn schedule_task() {

    log::Builder::from_env(log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();

    let secs = 2;
    let now = SystemTime::now();
    let target_time = now + Duration::from_secs(secs);

    log::debug!("Schedule task...");
    schedule_task_at(target_time, print_hello_task);
}