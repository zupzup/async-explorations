use async_task::waker_fn;
use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use std::thread;

fn main() {
    let other = Other(10);
    let v = block_on(Yield(Box::new(other)));
    println!("v: {}", v);
}

fn block_on<F: Future>(future: F) -> F::Output {
    let mut f = Box::pin(future);

    let thread = thread::current();
    let waker = waker_fn(move || thread.unpark());
    let ctx = &mut Context::from_waker(&waker);
    loop {
        match f.as_mut().poll(ctx) {
            Poll::Pending => thread::park(),
            Poll::Ready(v) => return v,
        }
    }
}

struct Yield(Box<dyn Future<Output = String> + Send + 'static + Unpin>);

impl Future for Yield {
    type Output = String;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<String> {
        match Box::pin(self.0.as_mut()).as_mut().poll(ctx) {
            Poll::Pending => {
                println!("wrapper not ready");
                Poll::Pending
            }
            Poll::Ready(v) => Poll::Ready(format!("is ready: {}", v)),
        }
        // IF we didn't have the Unpin constrain, this unsafe block would be needed
        // for projection, as only Pin<&mut self.0) implements Future - could also use
        // pin_project
        // match unsafe { self.map_unchecked_mut(|s| s.0.as_mut()) }.poll(ctx) {
        //     Poll::Pending => {
        //         println!("wrapper not ready");
        //         Poll::Pending
        //     }
        //     Poll::Ready(v) => Poll::Ready(format!("is ready: {}", v)),
        // }
    }
}

struct Other(u32);

impl Future for Other {
    type Output = String;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<String> {
        if self.0 == 0 {
            Poll::Ready("Other".to_string())
        } else {
            println!("not ready");
            self.0 -= 1;
            ctx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
