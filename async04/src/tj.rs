use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub fn try_join<A, B, AR, BR, E>(a: A, b: B) -> impl Future<Output = Result<(AR, BR), E>>
where
    A: Future<Output = Result<AR, E>>,
    B: Future<Output = Result<BR, E>>,
{
    TryJoin::Polling {
        a: State::Future(a),
        b: State::Future(b),
    }
}

enum State<F, T, E>
where
    F: Future<Output = Result<T, E>>,
{
    Future(F),
    Ok(T),
}

enum TryJoin<A, B, AR, BR, E>
where
    A: Future<Output = Result<AR, E>>,
    B: Future<Output = Result<BR, E>>,
{
    Polling {
        a: State<A, AR, E>,
        b: State<B, BR, E>,
    },
    Done,
}

impl<A, B, AR, BR, E> Future for TryJoin<A, B, AR, BR, E>
where
    A: Future<Output = Result<AR, E>>,
    B: Future<Output = Result<BR, E>>,
{
    type Output = Result<(AR, BR), E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let (a, b) = match this {
            Self::Polling { a, b } => (a, b),
            Self::Done => panic!("TryJoin future polled after completion"),
        };

        if let State::Future(fut) = a {
            if let Poll::Ready(res) = unsafe { Pin::new_unchecked(fut) }.poll(cx) {
                *a = State::Ok(res?);
            }
        }

        if let State::Future(fut) = b {
            if let Poll::Ready(res) = unsafe { Pin::new_unchecked(fut) }.poll(cx) {
                *b = State::Ok(res?);
            }
        }

        match (a, b) {
            (State::Ok(_), State::Ok(_)) => match std::mem::replace(this, Self::Done) {
                Self::Polling {
                    a: State::Ok(a),
                    b: State::Ok(b),
                } => Ok((a, b)).into(),
                _ => unreachable!(),
            },
            _ => Poll::Pending,
        }
    }
}
