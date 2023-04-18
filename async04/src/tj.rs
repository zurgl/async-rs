use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

struct TryJoin<A, B, AR, BR, E>
where
    A: Future<Output = Result<AR, E>>,
    B: Future<Output = Result<BR, E>>,
{
    a: A,
    b: B,
    a_res: Option<AR>,
    b_res: Option<BR>,
}

pub fn try_join<A, B, AR, BR, E>(a: A, b: B) -> impl Future<Output = Result<(AR, BR), E>>
where
    A: Future<Output = Result<AR, E>>,
    B: Future<Output = Result<BR, E>>,
{
    TryJoin {
        a,
        b,
        a_res: None,
        b_res: None,
    }
}

impl<A, B, AR, BR, E> Future for TryJoin<A, B, AR, BR, E>
where
    A: Future<Output = Result<AR, E>>,
    B: Future<Output = Result<BR, E>>,
{
    type Output = Result<(AR, BR), E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let (a, b) = unsafe {
            (
                Pin::new_unchecked(&mut this.a),
                Pin::new_unchecked(&mut this.b),
            )
        };

        if this.a_res.is_none() {
            if let Poll::Ready(res) = a.poll(cx) {
                match res {
                    Ok(x) => this.a_res = Some(x),
                    Err(e) => return Err(e).into(),
                }
            }
        }

        if this.b_res.is_none() {
            if let Poll::Ready(res) = b.poll(cx) {
                match res {
                    Ok(x) => this.b_res = Some(x),
                    Err(e) => return Err(e).into(),
                }
            }
        }

        if let (Some(_), Some(_)) = (&this.a_res, &this.b_res) {
            let a = this.a_res.take().unwrap();
            let b = this.b_res.take().unwrap();
            Ok((a, b)).into()
        } else {
            Poll::Pending
        }
    }
}
