use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{Poll, Context};

#[derive(Debug)]
enum FibState {
	Halted,
	Running,
}

#[derive(Debug)]
struct Fiber {
	state: FibState
}

impl FibState {
	fn waiter<'a>(&'a mut self)->Waiter<'a>{
		Waiter {fiber: self}
	}
}

struct Waiter<'a>{
	fiber: &'a mut Fiber,
}

impl<'a> Future for Waiter<'a>{
	type Output = ();

	fn poll(mut self: Pin<&mut Self>, ctx: &mut Context) -> RetType {
		match self.fiber.state {
		    State::Halted => {
		        self.fiber.state = State::Running;
		        Poll::Ready(())
		    }
		    State::Running => {
		        self.fiber.state = State::Halted;
		        Poll::Pending
		    }
		}	}
}

#[derive(Debug)]
struct Executor {
	fibers: VecDeque<Pin<Box<dyn Future<Output = ()>>>>,
}

impl Executor {
    fn new() -> Self {
        Executor {
            fibers: VecDeque::new(),
        }
    }

    fn push<C, F>(&mut self, closure: C)
    where
        F: Future<Output=()> + 'static,
        C: FnOnce(Fib) -> F,
    {
        let fib = Fiber { state: State::Running };
        self.fibers.push_back(Box::pin(closure(fiber)));
    }

    fn run(&mut self) {
        let waker = waker::create();
        let mut context = Context::from_waker(&waker);

        while let Some(mut fib) = self.fibers.pop_front() {
            match fib.as_mut().poll(&mut context) {
                Poll::Pending => {
                    self.fibers.push_back(fib);
                },
                Poll::Ready(()) => {},
            }
        }
    }
}
