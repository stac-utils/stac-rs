//! Walk [Stacs](Stac).

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
