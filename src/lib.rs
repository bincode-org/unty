#![no_std]
//! A crate that allows you to mostly-safely cast one type into another type.
//!
//! This is mostly useful for generic functions, e.g.
//!
//! ```
//! # use unty::*;
//! pub fn foo<S>(s: S) {
//!     if let Ok(a) = unsafe { unty::<S, u8>(s) } {
//!         println!("It is an u8 with value {a}");
//!     } else {
//!         println!("it is not an u8");
//!     }
//! }
//! foo(10u8); // will print "it is an u8"
//! foo("test"); // will print "it is not an u8"
//! ```
//!
//! This operation is still unsafe because it allows you to extend lifetimes. There currently is not a way to prevent this
//!
//! ```no_run
//! # fn foo<'a>(input: &'a str) {
//! # use unty::*;
//! if let Ok(str) = unsafe { unty::<&'a str, &'static str>(input) } {
//!     // the compiler may now light your PC on fire
//! }
//! # }
//! ```

use core::{any::TypeId, marker::PhantomData, mem};

/// Untypes your types. For documentation see the root of this crate.
///
/// # Safety
///
/// This should not be used with types with lifetimes.
pub unsafe fn unty<Src, Target: 'static>(x: Src) -> Result<Target, Src> {
    if type_equal::<Src, Target>() {
        let ptr = &x as *const Src as *const Target;
        mem::forget(x); // we're going to copy this, so don't run the destructor
        Ok(core::ptr::read(ptr))
    } else {
        Err(x)
    }
}

/// Checks to see if the two types are probably equal.
///
/// Note that this may give false positives if any of the types have lifetimes.
pub fn type_equal<Src, Target>() -> bool {
    non_static_type_id::<Src>() == non_static_type_id::<Target>()
}

// Code by dtolnay in a bincode issue:
// https://github.com/bincode-org/bincode/issues/665#issue-1903241159
fn non_static_type_id<T: ?Sized>() -> TypeId {
    trait NonStaticAny {
        fn get_type_id(&self) -> TypeId
        where
            Self: 'static;
    }

    impl<T: ?Sized> NonStaticAny for PhantomData<T> {
        fn get_type_id(&self) -> TypeId
        where
            Self: 'static,
        {
            TypeId::of::<T>()
        }
    }

    let phantom_data = PhantomData::<T>;
    NonStaticAny::get_type_id(unsafe {
        mem::transmute::<&dyn NonStaticAny, &(dyn NonStaticAny + 'static)>(&phantom_data)
    })
}

#[test]
fn test_double_drop() {
    use core::sync::atomic::{AtomicUsize, Ordering};
    #[derive(Debug)]
    struct Ty;
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    impl Drop for Ty {
        fn drop(&mut self) {
            COUNTER.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn foo<T: core::fmt::Debug>(t: T) {
        unsafe { unty::<T, Ty>(t) }.unwrap();
    }

    foo(Ty);
    assert_eq!(COUNTER.load(Ordering::Relaxed), 1);
}
