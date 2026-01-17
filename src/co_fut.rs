#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use core::{ffi::c_int, task::Poll};

use embassy_sync::waitqueue::AtomicWaker;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// [CoState] is the state of a [CoFut].
enum CoState {
    Starting,
    Pending,
    Finished,
}

/// [CoFut] is a [Future] that eventually holds the result of an asynchronous C function that makes
/// use of the `await` primitive.
pub struct CoFut {
    state: CoState,
    func: unsafe extern "C" fn() -> c_int,
    waker: &'static AtomicWaker,
    result: Option<c_int>,
}

impl CoFut {
    pub fn new(func: unsafe extern "C" fn() -> c_int, waker: &'static AtomicWaker) -> Self {
        Self {
            state: CoState::Starting,
            result: None,
            func,
            waker,
        }
    }
}

impl Future for CoFut {
    type Output = c_int;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        match self.state {
            CoState::Starting => {
                let result = unsafe { trampoline(false, Some(self.func)) };
                if result.status == PollStatus_READY {
                    return Poll::Ready(result.result);
                }

                self.waker.register(cx.waker());
                self.state = CoState::Pending;
                Poll::Pending
            }

            CoState::Pending => match unsafe { trampoline(true, Some(self.func)) } {
                CoPoll {
                    status: PollStatus_PENDING,
                    ..
                } => Poll::Pending,
                _ => {
                    self.state = CoState::Finished;
                    // INVARIANT: `func' has finished by this point, and we have its result.
                    Poll::Ready(self.result.unwrap())
                }
            },

            // INVARIANT: `func' has finished by this point, and we have its result.
            CoState::Finished => Poll::Ready(self.result.unwrap()),
        }
    }
}
