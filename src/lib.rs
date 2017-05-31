//! Box dynamically-sized types on stack
//! Requires nightly rust.
//!
//! Store or return trait-object and closure without heap allocation, and fallback to heap when thing goes too large.
//!
//! # Usage
//! First, add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! smallbox = "0.2"
//! ```
//!
//! Next, add this to your crate root:
//!
//! ```rust
//! extern crate smallbox;
//! ```
//!
//! Currently smallbox by default links to the standard library, but if you would
//! instead like to use this crate in a `#![no_std]` situation or crate, and want to 
//! opt out heap dependency and `SmallBox<T>` type, you can request this via:
//!
//! ```toml
//! [dependencies]
//! smallbox = { version = "0.2", default-features = false }
//! ```
//!
//! Enable `heap` feature for `#![no_std]` build to link `alloc` crate
//! and bring `SmallBox<T>` back.
//!
//! ```toml
//! [dependencies.smallbox]
//! version = "0.2"
//! default-features = false
//! features = ["heap"]
//! ```
//!
//!
//! # Feature Flags
//! The **arraydeque** crate has the following cargo feature flags:
//!
//! - `std`
//!   - Optional, enabled by default
//!   - Use libstd
//!
//!
//! - `heap`
//!   - Optional
//!   - Use heap fallback and include `SmallBox<T>` type, and link to `alloc` crate if `std`
//!     feature flag is opted out.
//!
//!
//! # Overview
//! This crate delivers two core type:
//!
//!  `StackBox<T>`: Represents a fixed-capacity allocation, and on stack stores dynamically-sized type.
//!    The `new` method on this type allows creating a instance from a concrete type,
//!    returning `Err(value)` if the instance is too large for the allocated region.
//!    So far, the fixed-capcity is about four words (4 * `sizeof(usize)`)
//!
//!  `SmallBox<T>`: Takes `StackBox<T>` as an varience, and fallback to `Box<T>` when type `T` is too large for `StackBox<T>`.
//!
//!
//! # Example
//! The simplest usage can be trait object dynamic-dispatch
//!
//! ```rust
//! use smallbox::StackBox;
//!
//! let val: StackBox<PartialEq<usize>> = StackBox::new(5usize).unwrap();
//!
//! assert!(*val == 5)
//! ```
//!
//! `Any` downcasting is also quite a good use.
//!
//! ```rust
//! use std::any::Any;
//! use smallbox::StackBox;
//!
//! let num: StackBox<Any> = StackBox::new(1234u32).unwrap();
//!
//! if let Some(num) = num.downcast_ref::<u32>() {
//!     assert_eq!(*num, 1234);
//! } else {
//!     unreachable!();
//! }
//! ```
//!
//! Another use case is to allow returning capturing closures without having to box them.
//!
//! ```rust
//! use smallbox::StackBox;
//!
//! fn make_closure(s: String) -> StackBox<Fn()->String> {
//!     StackBox::new(move || format!("Hello, {}", s)).ok().unwrap()
//! }
//!
//! let closure = make_closure("world!".to_owned());
//! assert_eq!(closure(), "Hello, world!");
//! ```
//!
//! `SmallBox<T>` is to eliminate heap alloction for small things, except that
//! the object is large enough to allocte.
//! In addition, the inner `StackBox<T>` or `Box<T>` can be moved out by explicit pattern match.
//!
//! ```rust
//! # #[cfg(feature = "heap")]
//! # {
//! use smallbox::SmallBox;
//!
//! let tiny: SmallBox<[u64]> = SmallBox::new([0; 2]);
//! let big: SmallBox<[u64]> = SmallBox::new([1; 8]);
//!
//! assert_eq!(tiny.len(), 2);
//! assert_eq!(big[7], 1);
//!
//! match tiny {
//!     SmallBox::Stack(val) => assert_eq!(*val, [0; 2]),
//!     _ => unreachable!()
//! }
//!
//! match big {
//!     SmallBox::Box(val) => assert_eq!(*val, [1; 8]),
//!     _ => unreachable!()
//! }
//! # }


//! ```

#![feature(unsize)]
#![feature(box_syntax)]
#![feature(unique)]
#![feature(used)]

#![cfg_attr(not(feature="std"), no_std)]
#![cfg_attr(all(feature="heap", not(feature="std")), feature(alloc))]

#[cfg(not(feature = "std"))]
extern crate core as std;
#[cfg(all(feature="heap", not(feature="std")))]
extern crate alloc;

pub mod space;
mod stackbox;
#[cfg(feature = "heap")]
mod smallbox;

pub use stackbox::StackBox;
#[cfg(feature = "heap")]
pub use smallbox::SmallBox;