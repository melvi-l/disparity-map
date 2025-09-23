use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};

pub fn block_on<F: Future>(mut future: F) -> F::Output {
    let waker = dummy_waker();
    let mut cx = Context::from_waker(&waker);
    let mut future = unsafe { Pin::new_unchecked(&mut future) };

    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(val) => return val,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

fn dummy_waker() -> Waker {
    struct Dummy;
    impl Wake for Dummy {
        fn wake(self: Arc<Self>) {}
    }
    Waker::from(Arc::new(Dummy))
}
