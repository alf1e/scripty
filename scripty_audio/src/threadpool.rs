use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::RecvError as TokioRecvError;

static THREADPOOL_SUBMIT: OnceCell<mpsc::SyncSender<Box<dyn FnOnce() + Send + Sync>>> =
    OnceCell::new();
static COMPLETED_JOBS: AtomicU64 = AtomicU64::new(0);

pub fn init_threadpool() {
    let pool = threadpool::Builder::new()
        .num_threads(num_cpus::get() / 2)
        .build();
    let (tx, rx) = mpsc::sync_channel(usize::MAX);
    THREADPOOL_SUBMIT
        .set(tx)
        .unwrap_or_else(|_| panic!("don't call `init_threadpool()` more than once"));

    std::thread::spawn(move || loop {
        match rx.recv() {
            Ok(rx) => pool.execute(move || {
                rx();
                COMPLETED_JOBS.fetch_add(1, Ordering::Relaxed);
            }),
            Err(_) => return,
        }
    });
}

pub async fn submit_job_async<T: 'static + Send + Sync>(
    f: Box<dyn FnOnce(oneshot::Sender<T>) + Send + Sync>,
) -> Result<T, TokioRecvError> {
    let (tx, rx) = oneshot::channel();

    THREADPOOL_SUBMIT
        .get()
        .expect("failed to fetch threadpool submitter")
        .send(Box::new(|| f(tx)))
        .unwrap_or_else(|_| panic!("call `init_threadpool()` before submitting jobs"));

    rx.await
}

#[inline]
pub fn get_completed_jobs() -> u64 {
    COMPLETED_JOBS.load(Ordering::Relaxed)
}