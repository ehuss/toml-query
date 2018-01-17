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

pub trait QueryExecutor {
    fn query<Q, T>(&mut self, q: &Q) -> Result<Q::Output>
        where Q: Query<T>;
}

impl QueryExecutor for Value {

    fn query<Q, T>(&mut self, q: &Q) -> Result<Q::Output>
        where Q: Query<T>
    {
        q.execute(self, None as Option<T>)
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

    #[test]
    fn compile_test_4() {
        use read::TomlValueReadExt;

        struct A;
        impl Query<()> for A {
            type Output = Option<Value>;

            fn execute(&self, t: &mut Value, p: Option<()>) -> Result<Self::Output> {
                t.read("foo").map(|o| o.map(Clone::clone))
            }
        }

        struct B;
        impl Query<Option<Value>> for B {
            type Output = Option<(Value, Value)>;
            fn execute(&self, t: &mut Value, p: Option<Option<Value>>) -> Result<Self::Output> {
                let v2 = t.read("bar")?;

                match p {
                    Some(Some(v1)) => match v2 {
                        Some(t) => Ok(Some((v1, t.clone()))),
                        None => Ok(None),
                    },

                    Some(None) => Ok(None),
                    None       => Ok(None)
                }
            }
        }

        let mut toml : Value = ::toml::from_str("foo = 1\nbar = 2").unwrap();
        let a                = A;
        let b                = B;
        let query            = a.chain(b);
        let res : Result<Option<(Value, Value)>> = toml.query(&query);

        match res.unwrap() {
            Some((Value::Integer(1), Value::Integer(2))) => assert!(true),
            Some((_, _)) => assert!(false, "Wrong Value types"),
            None => assert!(false, "No result"),
        }
    }
}
