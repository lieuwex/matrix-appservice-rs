use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

/// An ID being either a Matrix ID or an external ID for one object.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MappingId<'a, E, M>
where
    E: Clone + Eq + Hash + fmt::Display,
    M: Clone + Eq + Hash + fmt::Display + std::convert::AsRef<str>,
{
    External(&'a E),
    Matrix(&'a M),
}

/// Represents an object that has both a Matrix ID and an external ID.
pub trait Mappable {
    type MatrixType: Clone + Eq + Hash + fmt::Display + std::convert::AsRef<str>;
    type ExternalType: Clone + Eq + Hash + fmt::Display;

    fn get_matrix(&self) -> &Self::MatrixType;
    fn into_matrix(self) -> Self::MatrixType;
    fn get_external(&self) -> &Self::ExternalType;
    fn into_external(self) -> Self::ExternalType;
}

/// A map comparable to a `HashMap` which contains items that are `Mappable`.
/// The map keeps track of the mapping between both the external type and Matrix type and an
/// object.
#[derive(Debug, Clone)]
pub struct MappingDict<V: Mappable> {
    items: Vec<V>,
    external_to_index: HashMap<V::ExternalType, usize>,
    matrix_to_index: HashMap<V::MatrixType, usize>,
}

impl<V> MappingDict<V>
where
    V: Mappable,
{
    /// Create a new empty `MappingDict`.
    pub fn new() -> Self {
        Self {
            items: vec![],
            external_to_index: HashMap::new(),
            matrix_to_index: HashMap::new(),
        }
    }

    /// Create a new `MappingDict` consuming the given `Vec` of items.
    /// All items are put into the newly created map.
    ///
    /// This is more efficient than just calling `insert` yourself on an empty map, since this
    /// method will initialize the vector and hashmap with a starting capacpity, thus resulting in
    /// less allocations.
    pub fn from_vec(items: Vec<V>) -> Self {
        let mut res = Self {
            items: Vec::with_capacity(items.len()),
            matrix_to_index: HashMap::with_capacity(items.len()),
            external_to_index: HashMap::with_capacity(items.len()),
        };

        for item in items {
            res.insert(item);
        }

        res
    }

    /// Inserts the given `item` in the current `MappingDict`.
    /// Allocates if neccesary.
    ///
    /// Returns a mutable reference to the newly inserted item.
    pub fn insert(&mut self, item: V) -> &mut V {
        let index = self.items.len();

        self.matrix_to_index
            .insert((*item.get_matrix()).clone(), index);
        self.external_to_index
            .insert((*item.get_external()).clone(), index);
        self.items.push(item);

        &mut self.items[index]
    }

    /// Returns a reference to the item associated with the given `identifier`, or `None` if no
    /// such item exists.
    pub fn get(&self, identifier: MappingId<V::ExternalType, V::MatrixType>) -> Option<&V> {
        let index = match identifier {
            MappingId::Matrix(m) => self.matrix_to_index.get(m),
            MappingId::External(e) => self.external_to_index.get(e),
        };

        match index {
            None => None,
            Some(i) => self.items.get(*i),
        }
    }

    /// Returns a mutable reference to the item associated with the given `identifier`, or `None`
    /// if no such item exists.
    pub fn get_mut(
        &mut self,
        identifier: MappingId<V::ExternalType, V::MatrixType>,
    ) -> Option<&mut V> {
        let index = match identifier {
            MappingId::Matrix(m) => self.matrix_to_index.get(m),
            MappingId::External(e) => self.external_to_index.get(e),
        };

        match index {
            None => None,
            Some(i) => self.items.get_mut(*i),
        }
    }

    /// Returns whether or not this `MappingDict` contains an item associated with the given
    /// `identifier`.
    pub fn has(&self, identifier: MappingId<V::ExternalType, V::MatrixType>) -> bool {
        match identifier {
            MappingId::Matrix(m) => self.matrix_to_index.contains_key(m),
            MappingId::External(e) => self.external_to_index.contains_key(e),
        }
    }

    /// If this `MappingDict` contains an item associated with the given `identifier`, remove it
    /// and return the value that was contained in the `MappingDict`.
    /// If no such item exists, this function returns `None`.
    pub fn remove(&mut self, identifier: MappingId<V::ExternalType, V::MatrixType>) -> Option<V> {
        let index = match identifier {
            MappingId::Matrix(m) => self.matrix_to_index.remove(m),
            MappingId::External(e) => self.external_to_index.remove(e),
        };

        if let Some(id) = index {
            let item = self.items.remove(id);

            match identifier {
                MappingId::Matrix(_) => self.external_to_index.remove(item.get_external()),
                MappingId::External(_) => self.matrix_to_index.remove(item.get_matrix()),
            };

            Some(item)
        } else {
            None
        }
    }

    /// Get an iterator over references of the items contained in this `MappingDict`.
    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, V> {
        self.items.iter()
    }

    /// Get an iterator over mutable references of the items contained in this `MappingDict`.
    pub fn iter_mut<'a>(&'a mut self) -> std::slice::IterMut<'a, V> {
        self.items.iter_mut()
    }
}

impl<'a, T> IntoIterator for &'a MappingDict<T>
where
    T: Mappable,
{
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl<V> IntoIterator for MappingDict<V>
where
    V: Mappable,
{
    type Item = V;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}
