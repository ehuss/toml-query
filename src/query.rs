/// The Toml Query extensions

use std::marker::PhantomData;

use toml::Value;
use error::Result;

pub trait Query<Prev>
    where Prev: Sized,
          Self: Sized
{
    type Output: Sized;

    fn execute(&self, target: &mut Value, prev_result: Option<Prev>) -> Result<Self::Output>;

    fn chain<Q>(self, other: Q) -> Chain<Self, Prev, Q>
        where Q: Query<Self::Output>
    {
        Chain {
            first: self,
            _p: PhantomData,
            second: other
        }
    }
}

pub struct Chain<A, P, B>
    where A: Query<P>,
          B: Query<A::Output>,
          P: Sized
{
    first: A,
    _p: PhantomData<P>,
    second: B,
}

impl<A, P, B> Query<P> for Chain<A, P, B>
    where A: Query<P>,
          B: Query<A::Output>,
          P: Sized
{
    type Output = B::Output;

    fn execute(&self, target: &mut Value, prev_result: Option<P>) -> Result<Self::Output> {
        let p = self.first.execute(target, prev_result)?;
        self.second.execute(target, Some(p))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn compile_test_1() {
        struct A;
        impl<P> Query<P> for A {
            type Output = ();
            fn execute(&self, _t: &mut Value, p: Option<P>) -> Result<Self::Output> {
                Ok(())
            }
        }

        let a = A;
        let b = A;
        let c = A;
        let d = A;
        let chain = a.chain(b).chain(c).chain(d);
        let mut value = Value::Boolean(true);
        let res = chain.execute(&mut value, None as Option<()>).unwrap();
    }

    #[test]
    fn compile_test_2() {
        struct A;
        impl<P> Query<P> for A {
            type Output = ();
            fn execute(&self, _t: &mut Value, p: Option<P>) -> Result<Self::Output> {
                Ok(())
            }
        }

        struct B;
        impl<P> Query<P> for B {
            type Output = i32;
            fn execute(&self, _t: &mut Value, p: Option<P>) -> Result<Self::Output> {
                Ok(1)
            }
        }

        struct C;
        impl<P> Query<P> for C {
            type Output = f64;
            fn execute(&self, _t: &mut Value, p: Option<P>) -> Result<Self::Output> {
                Ok(1.0)
            }
        }

        struct D;
        impl<P> Query<P> for D {
            type Output = String;
            fn execute(&self, _t: &mut Value, p: Option<P>) -> Result<Self::Output> {
                Ok(String::from("Foo"))
            }
        }

        let a = A;
        let b = B;
        let c = C;
        let d = D;
        let chain = a.chain(b).chain(c).chain(d);
        let mut value = Value::Boolean(true);
        let res : String = chain.execute(&mut value, None as Option<()>).unwrap();
        assert_eq!(res, "Foo");
    }

    #[test]
    fn compile_test_3() {
        struct A;
        impl<P> Query<P> for A {
            type Output = ();
            fn execute(&self, _t: &mut Value, p: Option<P>) -> Result<Self::Output> {
                Ok(())
            }
        }

        struct B;
        impl<P> Query<P> for B {
            type Output = u32;
            fn execute(&self, _t: &mut Value, p: Option<P>) -> Result<Self::Output> {
                Ok(1)
            }
        }

        struct C;
        impl Query<u32> for C {
            type Output = f64;
            fn execute(&self, _t: &mut Value, p: Option<u32>) -> Result<Self::Output> {
                Ok(f64::from(p.unwrap_or(1)))
            }
        }

        struct D;
        impl Query<f64> for D {
            type Output = String;
            fn execute(&self, _t: &mut Value, p: Option<f64>) -> Result<Self::Output> {
                Ok(format!("f: {}", p.unwrap_or(1.0)))
            }
        }

        let a = A;
        let b = B;
        let c = C;
        let d = D;
        let chain = a.chain(b).chain(c).chain(d);
        let mut value = Value::Boolean(true);
        let res : String = chain.execute(&mut value, None as Option<()>).unwrap();
        assert_eq!(res, "f: 1");
    }
}
