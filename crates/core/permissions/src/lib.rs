// `#[async_trait]` marks the boxed future it generates `#[must_use]`, which it already is;
// the lint fires inside the macro expansion on every method of `PermissionQuery`.
#![allow(clippy::double_must_use)]

#[macro_use]
extern crate auto_ops;

#[macro_use]
extern crate async_trait;

mod r#impl;
mod models;
mod r#trait;

pub use r#impl::*;
pub use models::*;
pub use r#trait::*;

#[cfg(test)]
mod test;
