use std::sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
};
use std::thread;

pub struct ThreadPool {
    _handles: Vec<std::thread::JoinHandle<()>>,
    sender: Sender<Box<dyn FnMut() + Send>>,
}

impl ThreadPool {
    pub fn new(num_threads: u8) -> Self {
        let (tx, rx) = channel::<Box<dyn FnMut() + Send>>();
        let rx = Arc::new(Mutex::new(rx));
        let _handles = (0..num_threads)
            .map(|_| {
                let rx_handle = rx.clone();
                thread::spawn(move || loop {
                    let worker = match rx_handle.lock() {
                        Ok(receiver) => match receiver.recv() {
                            Ok(work) => Some(work),
                            Err(_) => break,
                        },
                        Err(_) => None,
                    };
                    match worker {
                        Some(mut worker) => {
                            println!("start..");
                            worker();
                            println!("end..");
                        }
                        None => break,
                    };
                })
            })
            .collect();

        Self {
            _handles,
            sender: tx,
        }
    }

    pub fn exec<F: FnMut() + Send + 'static>(&self, worker: F) {
        match self.sender.send(Box::new(worker)) {
            Ok(()) => (),
            Err(e) => println!("Error sending on sub-thread : {:?}", e),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::HashMap,
        sync::atomic::{AtomicU32, Ordering},
        time::Duration,
    };

    fn sleep(n: u64) {
        thread::sleep(Duration::from_secs(n));
    }

    fn get_users_from_db_1(users: Arc<Mutex<Vec<HashMap<&str, &str>>>>) {
        let mut user = HashMap::<&str, &str>::new();
        user.insert("name", "toto");
        users.lock().unwrap().push(user);
    }
    fn get_users_from_db_2(users: Arc<Mutex<Vec<HashMap<&str, &str>>>>) {
        let mut user = HashMap::<&str, &str>::new();
        user.insert("name", "titi");
        let mut user_2 = HashMap::<&str, &str>::new();
        user_2.insert("name", "tito");
        users.lock().unwrap().push(user);
        users.lock().unwrap().push(user_2);
    }

    #[test]
    fn it_works() {
        let pool = ThreadPool::new(5);

        let users = Arc::new(Mutex::new(Vec::<HashMap<&str, &str>>::new()));
        let users_clone_1 = users.clone();
        let users_clone_2 = users.clone();

        let numref = Arc::new(AtomicU32::new(0));
        let ref_clone = numref.clone();

        let incr = move || {
            ref_clone.fetch_add(1, Ordering::SeqCst);
        };

        pool.exec(|| {
            sleep(1);
            println!("Hi after sleep");
        });
        pool.exec(incr.clone());
        pool.exec(move || {
            sleep(2);
            get_users_from_db_1(users_clone_1.clone());
        });
        pool.exec(incr);
        pool.exec(move || get_users_from_db_2(users_clone_2.clone()));

        // here sleeping a bit to be sure every thread finished working  - not mandatory but usefull in the tests to get the logs
        sleep(3);
        let res = users.lock().unwrap();
        assert_eq!(res.len(), 3);
        assert_eq!(numref.load(Ordering::SeqCst), 2);
    }
}
