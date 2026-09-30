/// A lightweight, contiguous vector-backed map.
///
/// This type provides convenient access to key-value metadata stored within POD5 records.
/// Internally, entries are stored in insertion order in a contiguous vector,
/// and lookups perform a linear search.
///
/// Keys are not required to be unique.
/// When duplicate keys are present, methods such as [get] return the first matching entry.
pub struct FlatMap<'a> {
    data: Vec<(&'a str, &'a str)>,
}

impl<'a> FlatMap<'a> {
    /// Creates a new `FlatMap` from a vector of key-value tuples.
    pub fn new(data: Vec<(&'a str, &'a str)>) -> Self {
        Self {
            data
        }
    }

    /// Returns the value corresponding to the supplied key.
    ///
    /// If multiple identical keys exist, this returns the value of the first matching pair.
    pub fn get(&self, key: &str) -> Option<&'a str> {
        self.data
            .iter()
            .find(|(a, _)| *a == key)
            .map(|(_, b)| *b)
    }

    /// Returns `true` if at least one matching key exists.
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.iter().any(|(a, _)| a == &key)
    }

    /// Returns `true` if the map contains duplicate keys.
    pub fn has_duplicate_keys(&self) -> bool {
        let data = &self.data;
        let mut indices: Vec<usize> = (0..data.len()).collect();
        indices.sort_unstable_by_key(|&i| data[i].0);
        indices.windows(2).any(|ij| data[ij[0]].0 == data[ij[1]].0)
    }

    /// Returns an iterator over the keys in insertion order.
    pub fn keys(&self) -> impl Iterator<Item = &'a str> {
        self.data.iter().map(|(a, _)| a.clone())
    }

    /// Returns an iterator over the values in insertion order.
    pub fn values(&self) -> impl Iterator<Item = &'a str> {
        self.data.iter().map(|(_, value)| value.clone())
    }

    /// Returns the number of elements in the map.
    /// Duplicate keys are counted as distinct entries.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns an iterator over the key-value pairs in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &(&'a str, &'a str)> {
        self.data.iter()
    }
}