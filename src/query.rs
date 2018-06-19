/// The Toml Query extensions
///
/// The Query extension gives an interface where the user can build a chain of queries which shall
/// be executed on a toml document.
///
/// The execution of the query can be implemented in different ways, for example
///
/// 1. errors beeing collected, but all queries are applied to the document
/// 1. the first error halts the execution
/// 1. something in between
///
/// as the implementation of the `Query` trait defines how it is executed.
///
/// More details are described in the documentation of the trait and types provided by this module.
///

use std::marker::PhantomData;

use toml::Value;

/// The Query trait
///
/// The Query trait describes an object which can be used to query/alter a TOML document. Thus, it
/// gets a mutable reference to the _whole_ document.
/// In addition, it gets the result of the previous query which was applied to the document. If the
/// object is the first Query to be applied, it gets `None`.
///
/// The implementation of the `execute` function defines how the execution works and how errors are
/// collected or populated.
///
/// # Chaining
///
/// The `Query::chain` function contains a default implementation for chaining two `Query`able
/// objects together. The first object (the object the function is called on) gets called first in
/// the chain.
///
/// See also: Documentation of the `Chain` type.
///
pub trait Query<Prev, E>
    where Prev: Sized,
          Self: Sized
{
    type Output: Sized;

    fn execute(&self, target: &mut Value, prev_result: Option<Prev>) -> Result<Self::Output, E>;

    fn chain<Q>(self, other: Q) -> Chain<Self, Prev, Q, E>
        where Q: Query<Self::Output, E>
    {
        Chain {
            first: self,
            _p: PhantomData,
            second: other,
            _e: PhantomData,
        }
    }
}

/// The `Chain` type
///
/// A type which can be used to chain two `Query`able objects. As `Chain` implements `Query` as
/// well, it can be chained as well.
///
/// The implementation of `Query::execute` on the `Chain` type returns (the error) if the first
/// `Query` object returns an error. The second one is not executed in this case.
///
pub struct Chain<A, P, B, E>
    where A: Query<P, E>,
          B: Query<A::Output, E>,
          P: Sized
{
    first: A,
    _p: PhantomData<P>,
    second: B,
    _e: PhantomData<E>,
}

impl<A, P, B, E> Query<P, E> for Chain<A, P, B, E>
    where A: Query<P, E>,
          B: Query<A::Output, E>,
          P: Sized
{
    type Output = B::Output;

    fn execute(&self, target: &mut Value, prev_result: Option<P>) -> Result<Self::Output, E> {
        let p = self.first.execute(target, prev_result)?;
        self.second.execute(target, Some(p))
    }
}

/// A trait for execution
///
/// This trait defines how the overall execution of the `Query` object(s) should be invoked.
///
/// It is implemented for `toml::Value` which simply calls `Query::execute` for the TOML document
/// with the `Query` object that is passed.
///
pub trait QueryExecutor {
    fn query<Q, T, E>(&mut self, q: &Q) -> Result<Q::Output, E>
        where Q: Query<T, E>;
}

impl QueryExecutor for Value {

    fn query<Q, T, E>(&mut self, q: &Q) -> Result<Q::Output, E>
        where Q: Query<T, E>
    {
        q.execute(self, None as Option<T>)
    }

}

/// A custom executor for executing queries on a document, which resets the document if an error
/// occured during query execution
///
/// # Warning
///
/// When calling `ResetExecutor::new()`, the passed document is cloned. This may introduce a lot of
/// overhead, but the clone is necessary for resetting the document in case of error.
///
/// This is a naiive implementation of a "resetting" query infrastructure and one may implement a
/// more sophisticated solution for this problem by implementing custom `Query` objects that can
/// handle rollback more efficiently.
///
struct ResetExecutor<'doc>(&'doc mut Value, Value);

impl<'doc> ResetExecutor<'doc> {
    pub fn new(doc: &'doc mut Value) -> Self {
        ResetExecutor(doc, doc.clone())
    }
}

impl<'doc> QueryExecutor for ResetExecutor<'doc> {
    fn query<Q, T, E>(&mut self, q: &Q) -> Result<Q::Output, E>
        where Q: Query<T, E>
    {
        q.execute(&mut self.0, None as Option<T>).map_err(|e| {
            *self.0 = self.1;
            e
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    type Result<T> = ::std::result::Result<T, ()>;

    #[test]
    fn compile_test_1() {
        struct A;
        impl<P> Query<P, ()> for A {
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
        impl<P> Query<P, ()> for A {
            type Output = ();
            fn execute(&self, _t: &mut Value, p: Option<P>) -> Result<Self::Output> {
                Ok(())
            }
        }

        struct B;
        impl<P> Query<P, ()> for B {
            type Output = i32;
            fn execute(&self, _t: &mut Value, p: Option<P>) -> Result<Self::Output> {
                Ok(1)
            }
        }

        struct C;
        impl<P> Query<P, ()> for C {
            type Output = f64;
            fn execute(&self, _t: &mut Value, p: Option<P>) -> Result<Self::Output> {
                Ok(1.0)
            }
        }

        struct D;
        impl<P> Query<P, ()> for D {
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
        impl<P> Query<P, ()> for A {
            type Output = ();
            fn execute(&self, _t: &mut Value, p: Option<P>) -> Result<Self::Output> {
                Ok(())
            }
        }

        struct B;
        impl<P> Query<P, ()> for B {
            type Output = u32;
            fn execute(&self, _t: &mut Value, p: Option<P>) -> Result<Self::Output> {
                Ok(1)
            }
        }

        struct C;
        impl Query<u32, ()> for C {
            type Output = f64;
            fn execute(&self, _t: &mut Value, p: Option<u32>) -> Result<Self::Output> {
                Ok(f64::from(p.unwrap_or(1)))
            }
        }

        struct D;
        impl Query<f64, ()> for D {
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
        use error::Error;

        type Result<T> = ::std::result::Result<T, Error>;

        struct A;
        impl Query<(), Error> for A {
            type Output = Option<Value>;

            fn execute(&self, t: &mut Value, p: Option<()>) -> Result<Self::Output> {
                t.read("foo").map(|o| o.map(Clone::clone))
            }
        }

        struct B;
        impl Query<Option<Value>, Error> for B {
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
