//! Abstract `Component` trait and some common storage stratedy.

use std::ptr;
use std::any::Any;
use std::collections::HashMap;

use utils::handle::HandleIndex;

use super::Entity;
use super::bitset::DynamicBitSet;

/// Abstract component trait with associated storage type.
pub trait Component: Any + 'static
where
    Self: Sized,
{
    type Arena: ComponentArena<Self> + Any + Send + Sync;

    fn drop(&mut Self::Arena, _: Entity) {}
}

/// Declare a struct as component, and specify the storage strategy. Internally, this
/// macro will impl a internal trait `Component` to provide some useful methods and hints.
#[macro_export]
macro_rules! declare_component {
    ( $CMP:ident, $STORAGE:ident ) => {
        impl $crate::ecs::Component for $CMP {
            type Arena = $STORAGE<$CMP>;
        }
    };
}

/// Traits used to implement a standart/basic storage for components. Choose your
/// components storage layout and strategy by declaring different `ComponentArena`
/// with corresponding component.
pub trait ComponentArena<T>
where
    T: Component,
{
    /// Creates a new `ComponentArena<T>`. This is called when you register a new component
    /// type within the world.
    fn new() -> Self;

    /// Returns a reference to the value corresponding to the `HandleIndex`,
    fn get(&self, HandleIndex) -> Option<&T>;

    /// Returns a mutable reference to the value at the `HandleIndex`, without doing
    /// bounds checking.
    unsafe fn get_unchecked(&self, HandleIndex) -> &T;

    /// Returns a mutable reference to the value corresponding to the `HandleIndex`,
    fn get_mut(&mut self, HandleIndex) -> Option<&mut T>;

    /// Returns a mutable reference to the value at the `HandleIndex`, without doing
    /// bounds checking.
    unsafe fn get_unchecked_mut(&mut self, HandleIndex) -> &mut T;

    /// Inserts new data for a given `HandleIndex`,
    fn insert(&mut self, HandleIndex, T) -> Option<T>;

    /// Removes and returns the data associated with an `HandleIndex`.
    fn remove(&mut self, HandleIndex) -> Option<T>;
}

/// `HashMap` based storage which are best suited for rare components.
pub struct HashMapArena<T>
where
    T: Component,
{
    values: HashMap<HandleIndex, T>,
}

impl<T> ComponentArena<T> for HashMapArena<T>
where
    T: Component,
{
    fn new() -> Self {
        HashMapArena {
            values: HashMap::new(),
        }
    }

    fn get(&self, id: HandleIndex) -> Option<&T> {
        self.values.get(&id)
    }

    unsafe fn get_unchecked(&self, id: HandleIndex) -> &T {
        &self.values[&id]
    }

    fn get_mut(&mut self, id: HandleIndex) -> Option<&mut T> {
        self.values.get_mut(&id)
    }

    unsafe fn get_unchecked_mut(&mut self, id: HandleIndex) -> &mut T {
        self.values.get_mut(&id).unwrap()
    }

    fn insert(&mut self, id: HandleIndex, v: T) -> Option<T> {
        self.values.insert(id, v)
    }

    fn remove(&mut self, id: HandleIndex) -> Option<T> {
        self.values.remove(&id)
    }
}

/// Vec based storage, supposed to have maximum performance for the components
/// mostly present in entities.
pub struct VecArena<T>
where
    T: Component + ::std::fmt::Debug,
{
    mask: DynamicBitSet,
    values: Vec<T>,
}

impl<T> ComponentArena<T> for VecArena<T>
where
    T: Component + ::std::fmt::Debug,
{
    fn new() -> Self {
        VecArena {
            mask: DynamicBitSet::new(),
            values: Vec::new(),
        }
    }

    fn get(&self, id: HandleIndex) -> Option<&T> {
        if self.mask.contains(id as usize) {
            self.values.get(id as usize)
        } else {
            None
        }
    }

    unsafe fn get_unchecked(&self, id: HandleIndex) -> &T {
        self.values.get_unchecked(id as usize)
    }

    fn get_mut(&mut self, id: HandleIndex) -> Option<&mut T> {
        if self.mask.contains(id as usize) {
            self.values.get_mut(id as usize)
        } else {
            None
        }
    }

    unsafe fn get_unchecked_mut(&mut self, id: HandleIndex) -> &mut T {
        self.values.get_unchecked_mut(id as usize)
    }

    fn insert(&mut self, id: HandleIndex, v: T) -> Option<T> {
        unsafe {
            let len = self.values.len();
            if id as usize >= len {
                self.values.reserve(id as usize + 1 - len);
                self.values.set_len(id as usize + 1);
            }

            // Write the value without reading or dropping
            // the (currently uninitialized) memory.
            let value = if self.mask.contains(id as usize) {
                Some(ptr::read(self.get_unchecked(id)))
            } else {
                self.mask.insert(id as usize);
                None
            };

            ptr::write(self.values.get_unchecked_mut(id as usize), v);
            value
        }
    }

    fn remove(&mut self, id: HandleIndex) -> Option<T> {
        unsafe {
            if self.mask.contains(id as usize) {
                self.mask.remove(id as usize);
                Some(ptr::read(self.get_unchecked(id)))
            } else {
                None
            }
        }
    }
}

impl<T> Drop for VecArena<T>
where
    T: Component + ::std::fmt::Debug,
{
    fn drop(&mut self) {
        unsafe {
            for i in self.mask.iter() {
                ptr::read(self.values.get_unchecked(i));
            }

            self.values.set_len(0);
            self.mask.clear();
        }
    }
}

#[cfg(test)]
mod test {}
