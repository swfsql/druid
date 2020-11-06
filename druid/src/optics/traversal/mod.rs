#[allow(clippy::module_inception)]
mod traversal;
#[doc(hidden)]
pub use traversal::Traversal;
pub use traversal::{Then, VecTraversal};
