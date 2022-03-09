//! Configurable iteration over the [Objects](crate::Object) in a [Stac].
//!
//! This module provides two structures, [BorrowedWalk] and [OwnedWalk], that iterate over every [Object](crate::Object) in a [Stac].
//! As their names imply, `BorrowedWalk` holds a mutable reference to a `Stac`, while `OwnedWalk` consumes the `Stac`.
//! They are created by the [Stac::walk] and [Stac::into_walk] methods, respectively:
//!
//! ```
//! # use stac::{Stac, Catalog};
//! let (mut stac, root) = Stac::new(Catalog::new("root")).unwrap();
//!
//! // Mutably borrows `stac`.
//! let _ = stac.walk(root).collect::<Result<Vec<_>, _>>().unwrap();
//!
//! // Consumes `stac`.
//! let _ = stac.into_walk(root).collect::<Result<Vec<_>, _>>().unwrap();
//! ```
//!
//! # Examples
//!
//! By default, a walk iterates over `Result<Handle>`.
//! This can be useful to, e.g., count the number of objects in a `Stac` tree.
//! It has the side effect of resolving every object in the `Stac`.
//!
//! ```
//! # use stac::{Stac, Catalog};
//! let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
//! let handles = stac.walk(root).collect::<Result<Vec<_>, _>>().unwrap();
//! assert_eq!(handles.len(), 6);
//! ```
//!
//! This basic behavior isn't useful for modifying or querying the `Stac`, since you can't operate on the `Stac` while you are iterating over it.
//! This example will not compile:
//!
//! ```compile_fail
//! # use stac::{Stac, Catalog};
//! # let (mut stac, root) = Stac::new(Catalog::new("root")).unwrap();
//! for result in stac.walk(root) {
//!     let handle = result.unwrap();
//!     let object = stac.get(handle).unwrap(); // <- can't do this!
//! }
//! ```
//!
//! To do things during iteration, you need to use `visit`.
//!
//! ## Visit
//!
//! Both walk structures are generic over `F` and `T`, where `F: FnMut(&mut Stac<r>, Handle) -> Result<T>`.
//! This `F` is a `visit` function that is called each iteration of the walk.
//! You can set your own `visit` function to do things while you are visiting that member of the tree:
//!
//! ```
//! # use stac::{Stac, Catalog};
//! let (mut stac, root) = Stac::new(Catalog::new("root")).unwrap();
//! let nothing = stac
//!     .walk(root)
//!     .visit(|stac, handle| stac.get(handle).map(|object| {
//!         println!("id={}", object.id());
//!     }))
//!     .collect::<Result<Vec<()>, _>>()
//!     .unwrap();
//! ```
//!
//! You'll notice in the above example we didn't return anything from our `visit` function, and so the type of object returned by the walk iterator changed as well.
//! This can be useful to collect attributes from a `Stac`:
//!
//! ```
//! # use stac::{Stac, Catalog};
//! let (mut stac, root) = Stac::new(Catalog::new("root")).unwrap();
//! let ids = stac
//!     .walk(root)
//!     .visit(|stac, handle| stac.get(handle).map(|object| {
//!         object.id().to_string()
//!     }))
//!     .collect::<Result<Vec<String>, _>>()
//!     .unwrap();
//! ```
//!
//! The `stac` argument to the `visit` function is a mutable reference, so you can modify the `Stac`.
//! In fact, we already have in the above examples, because [Stac::get] requires a mutable reference because it might have to resolve the object by reading it.
//! You can also modify the tree structure itself.
//! The `visit` function is called _before_ adding the current object's children to the iteration queue, and so you can use the `visit` function to change the iterations itself.
//! For example, let's add a single child item to the root:
//!
//! ```
//! use stac::{Read, Item, Stac, Catalog, Handle, Result};
//!
//! fn add_item<R: Read>(stac: &mut Stac<R>, handle: Handle) -> Result<Handle> {
//!     if stac.root() == handle {
//!         stac.add_child(handle, Item::new("an-item"));
//!     }
//!     Ok(handle)
//! }
//!
//! let (mut stac, root) = Stac::new(Catalog::new("root")).unwrap();
//! let handles = stac
//!     .walk(root)
//!     .visit(add_item)
//!     .collect::<Result<Vec<Handle>>>()
//!     .unwrap();
//! assert_eq!(handles.len(), 2);
//! assert_eq!(handles[0], root);
//! assert_eq!(handles[1], stac.children(root)[0])
//! ```
//!
//! ## The `Walk` trait
//!
//! Both [BorrowedWalk] and [OwnedWalk] implement [Walk], which provides methods to modify walk itself.
//! For example, you can traverse depth-first instead of the default breadth-first.
//! Notice how the `Walk` trait needs to be brought into scope to use the method:
//!
//! ```
//! use stac::{Stac, Catalog, Walk};
//! let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
//! let handles = stac.walk(root).depth_first().collect::<Result<Vec<_>, _>>().unwrap();
//! ```
//!
//! See the [Walk] trait documentation for more configuration options.

use super::{Handle, Stac};
use crate::{Read, Result};
use std::collections::VecDeque;

/// Walk
pub trait Walk: Sized {
    /// Options mut
    fn options_mut(&mut self) -> &mut Options;

    /// Walk depth-first instead of breadth first.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Stac, Walk};
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// let ids = stac
    ///     .walk(root)
    ///     .depth_first()
    ///     .visit(|stac, handle| {
    ///         stac.get(handle).map(|object| String::from(object.id()))
    ///     });
    /// for result in ids {
    ///     let id = result.unwrap();
    ///     println!("{}", id);
    /// }
    /// ```
    fn depth_first(mut self) -> Self {
        self.options_mut().depth_first = true;
        self
    }

    /// Skip items while walking the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Stac, Walk};
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// let ids = stac
    ///     .walk(root)
    ///     .skip_items()
    ///     .visit(|stac, handle| {
    ///         stac.get(handle).map(|object| String::from(object.id()))
    ///     })
    ///     .collect::<Result<Vec<_>, _>>()
    ///     .unwrap();
    /// assert_eq!(ids.len(), 4);
    /// ```
    fn skip_items(mut self) -> Self {
        self.options_mut().strategy = Strategy::SkipItems;
        self
    }

    /// Only stop at items when walking the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Stac, Walk};
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// let ids = stac
    ///     .walk(root)
    ///     .items_only()
    ///     .visit(|stac, handle| {
    ///         stac.get(handle).map(|object| String::from(object.id()))
    ///     })
    ///     .collect::<Result<Vec<_>, _>>()
    ///     .unwrap();
    /// assert_eq!(ids.len(), 2);
    /// ```
    fn items_only(mut self) -> Self {
        self.options_mut().strategy = Strategy::ItemsOnly;
        self
    }
}

/// An iterator over a [Stac's](Stac) [Handles](Handle).
#[derive(Debug)]
pub struct BorrowedWalk<'a, R: Read, F, T>
where
    F: FnMut(&mut Stac<R>, Handle) -> Result<T>,
{
    handles: VecDeque<Handle>,
    stac: &'a mut Stac<R>,
    visit: F,
    options: Options,
}

/// An owned walk over a [Stac].
#[derive(Debug)]
pub struct OwnedWalk<R: Read, F, T>
where
    F: FnMut(&mut Stac<R>, Handle) -> Result<T>,
{
    handles: VecDeque<Handle>,
    stac: Stac<R>,
    visit: F,
    options: Options,
}

/// Walk options
#[derive(Debug)]
pub struct Options {
    depth_first: bool,
    strategy: Strategy,
}

/// Walk strategy
#[derive(Debug)]
pub enum Strategy {
    /// Skip
    SkipItems,
    /// Items
    ItemsOnly,
    /// All
    All,
}

impl<R: Read> Stac<R> {
    /// Walk it
    pub fn walk(
        &mut self,
        handle: Handle,
    ) -> BorrowedWalk<'_, R, impl FnMut(&mut Stac<R>, Handle) -> Result<Handle>, Handle> {
        let mut handles = VecDeque::new();
        handles.push_front(handle);
        BorrowedWalk {
            handles,
            stac: self,
            visit: |_, handle| Ok(handle),
            options: Options::default(),
        }
    }

    /// Into walk it
    pub fn into_walk(
        self,
        handle: Handle,
    ) -> OwnedWalk<R, impl FnMut(&mut Stac<R>, Handle) -> Result<Handle>, Handle> {
        let mut handles = VecDeque::new();
        handles.push_front(handle);
        OwnedWalk {
            handles,
            stac: self,
            visit: |_, handle| Ok(handle),
            options: Options::default(),
        }
    }
}

impl<'a, R: Read, F, T> BorrowedWalk<'a, R, F, T>
where
    F: FnMut(&mut Stac<R>, Handle) -> Result<T>,
{
    /// Returns a new `Walk` with the provided `visit` function.
    pub fn visit<U>(
        self,
        visit: impl FnMut(&mut Stac<R>, Handle) -> Result<U>,
    ) -> BorrowedWalk<'a, R, impl FnMut(&mut Stac<R>, Handle) -> Result<U>, U> {
        BorrowedWalk {
            handles: self.handles,
            stac: self.stac,
            visit,
            options: self.options,
        }
    }
}

impl<R: Read, F, T> Walk for BorrowedWalk<'_, R, F, T>
where
    F: FnMut(&mut Stac<R>, Handle) -> Result<T>,
{
    fn options_mut(&mut self) -> &mut Options {
        &mut self.options
    }
}

impl<R: Read, F, T> Iterator for BorrowedWalk<'_, R, F, T>
where
    F: FnMut(&mut Stac<R>, Handle) -> Result<T>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        walk(
            &mut self.handles,
            &mut self.stac,
            &mut self.visit,
            &self.options,
        )
    }
}

impl<R: Read, F, T> OwnedWalk<R, F, T>
where
    F: FnMut(&mut Stac<R>, Handle) -> Result<T>,
{
    /// Returns a new `Walk` with the provided `visit` function.
    pub fn visit<U>(
        self,
        visit: impl FnMut(&mut Stac<R>, Handle) -> Result<U>,
    ) -> OwnedWalk<R, impl FnMut(&mut Stac<R>, Handle) -> Result<U>, U> {
        OwnedWalk {
            handles: self.handles,
            stac: self.stac,
            visit,
            options: self.options,
        }
    }
}

impl<R: Read, F, T> Walk for OwnedWalk<R, F, T>
where
    F: FnMut(&mut Stac<R>, Handle) -> Result<T>,
{
    fn options_mut(&mut self) -> &mut Options {
        &mut self.options
    }
}

impl<R: Read, F, T> Iterator for OwnedWalk<R, F, T>
where
    F: FnMut(&mut Stac<R>, Handle) -> Result<T>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        walk(
            &mut self.handles,
            &mut self.stac,
            &mut self.visit,
            &self.options,
        )
    }
}

impl Default for Options {
    fn default() -> Options {
        Options {
            depth_first: false,
            strategy: Strategy::All,
        }
    }
}

fn walk<R, T>(
    handles: &mut VecDeque<Handle>,
    stac: &mut Stac<R>,
    mut visit: impl FnMut(&mut Stac<R>, Handle) -> Result<T>,
    options: &Options,
) -> Option<Result<T>>
where
    R: Read,
{
    if let Some(handle) = handles.pop_front() {
        if let Err(err) = stac.ensure_resolved(handle) {
            handles.clear();
            Some(Err(err))
        } else {
            match (visit)(stac, handle) {
                Ok(value) => {
                    let mut children = VecDeque::new();
                    for &child in &stac.node(handle).children {
                        if !(matches!(options.strategy, Strategy::SkipItems) && stac.is_item(child))
                        {
                            if options.depth_first {
                                children.push_front(child);
                            } else {
                                children.push_back(child);
                            }
                        }
                    }
                    if options.depth_first {
                        for child in children {
                            handles.push_front(child);
                        }
                    } else {
                        handles.extend(children)
                    }
                    if !(matches!(options.strategy, Strategy::ItemsOnly) && !stac.is_item(handle)) {
                        Some(Ok(value))
                    } else {
                        walk(handles, stac, visit, options)
                    }
                }
                Err(err) => {
                    handles.clear();
                    Some(Err(err))
                }
            }
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::Walk;
    use crate::Stac;

    #[test]
    fn walk() {
        let (mut stac, handle) = Stac::read("data/catalog.json").unwrap();
        let ids = stac
            .walk(handle)
            .visit(|stac, handle| stac.get(handle).map(|object| object.id().to_string()))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            ids,
            vec![
                "examples",
                "extensions-collection",
                "sentinel-2",
                "sentinel-2",
                "CS3-20160503_132131_08",
                "proj-example",
            ]
        )
    }

    #[test]
    fn walk_depth_first() {
        let (mut stac, handle) = Stac::read("data/catalog.json").unwrap();
        let ids = stac
            .walk(handle)
            .visit(|stac, handle| stac.get(handle).map(|object| object.id().to_string()))
            .depth_first()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            ids,
            vec![
                "examples",
                "extensions-collection",
                "proj-example",
                "sentinel-2",
                "sentinel-2",
                "CS3-20160503_132131_08",
            ]
        )
    }

    #[test]
    fn walk_remove() {
        let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
        let count = stac
            .walk(root)
            .visit(|stac, handle| {
                if handle != root {
                    let _ = stac.remove(handle)?;
                    Ok(())
                } else {
                    Ok(())
                }
            })
            .count();
        assert_eq!(count, 5);
        assert!(stac.find(root, |o| o.is_collection()).unwrap().is_none());
        assert!(stac.find(root, |o| o.is_item()).unwrap().is_none());
    }
}
