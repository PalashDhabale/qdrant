use std::borrow::Cow;
use std::collections::HashMap;

use sparse::common::sparse_vector::SparseVector;

use super::tiny_map;
use super::vectors::{Vector, VectorElementType, VectorRef};
use crate::types::Distance;

type CowKey<'a> = Cow<'a, str>;

#[derive(Clone, PartialEq, Debug)]
pub enum CowValue<'a> {
    Dense(Cow<'a, [VectorElementType]>),
    Sparse(Cow<'a, SparseVector>),
}

impl<'a> Default for CowValue<'a> {
    fn default() -> Self {
        CowValue::Dense(Cow::Owned(Vec::new()))
    }
}

type TinyMap<'a> = tiny_map::TinyMap<CowKey<'a>, CowValue<'a>>;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct NamedVectors<'a> {
    map: TinyMap<'a>,
}

impl<'a> CowValue<'a> {
    pub fn to_owned(self) -> Vector {
        match self {
            CowValue::Dense(v) => Vector::Dense(v.into_owned()),
            CowValue::Sparse(v) => Vector::Sparse(v.into_owned()),
        }
    }

    pub fn as_vec_ref(&self) -> VectorRef {
        match self {
            CowValue::Dense(v) => VectorRef::Dense(v.as_ref()),
            CowValue::Sparse(v) => VectorRef::Sparse(v.as_ref()),
        }
    }
}

impl<'a> NamedVectors<'a> {
    pub fn from_ref(key: &'a str, value: VectorRef<'a>) -> Self {
        let mut map = TinyMap::new();
        map.insert(
            Cow::Borrowed(key),
            match value {
                VectorRef::Dense(v) => CowValue::Dense(Cow::Borrowed(v)),
                VectorRef::Sparse(v) => CowValue::Sparse(Cow::Borrowed(v)),
            },
        );
        Self { map }
    }

    pub fn from<const N: usize>(arr: [(String, Vec<VectorElementType>); N]) -> Self {
        NamedVectors {
            map: arr
                .into_iter()
                .map(|(k, v)| (CowKey::from(k), CowValue::Dense(Cow::Owned(v))))
                .collect(),
        }
    }

    pub fn from_map(map: HashMap<String, Vec<VectorElementType>>) -> Self {
        Self {
            map: map
                .into_iter()
                .map(|(k, v)| (CowKey::from(k), CowValue::Dense(Cow::Owned(v))))
                .collect(),
        }
    }

    pub fn from_map_ref(map: &'a HashMap<String, Vec<VectorElementType>>) -> Self {
        Self {
            map: map
                .iter()
                .map(|(k, v)| (CowKey::from(k), CowValue::Dense(Cow::Borrowed(v))))
                .collect(),
        }
    }

    pub fn insert(&mut self, name: String, vector: Vector) {
        self.map.insert(
            CowKey::Owned(name),
            match vector {
                Vector::Dense(v) => CowValue::Dense(Cow::Owned(v)),
                Vector::Sparse(v) => CowValue::Sparse(Cow::Owned(v)),
            },
        );
    }

    pub fn insert_ref(&mut self, name: &'a str, vector: VectorRef<'a>) {
        self.map.insert(
            CowKey::Borrowed(name),
            match vector {
                VectorRef::Dense(v) => CowValue::Dense(Cow::Borrowed(v)),
                VectorRef::Sparse(v) => CowValue::Sparse(Cow::Borrowed(v)),
            },
        );
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.map.iter().map(|(k, _)| k.as_ref())
    }

    pub fn into_owned_map(self) -> HashMap<String, Vector> {
        self.map
            .into_iter()
            .map(|(k, v)| (k.into_owned(), v.to_owned()))
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, VectorRef<'_>)> {
        self.map.iter().map(|(k, v)| (k.as_ref(), v.as_vec_ref()))
    }

    pub fn get(&self, key: &str) -> Option<VectorRef<'_>> {
        self.map.get(key).map(|v| v.as_vec_ref())
    }

    pub fn preprocess<F>(&mut self, distance_map: F)
    where
        F: Fn(&str) -> Distance,
    {
        for (name, vector) in self.map.iter_mut() {
            let distance = distance_map(name);
            match vector {
                CowValue::Dense(v) => {
                    let preprocessed_vector = distance.preprocess_vector(v.to_vec());
                    *vector = CowValue::Dense(Cow::Owned(preprocessed_vector))
                }
                CowValue::Sparse(_) => {}
            }
        }
    }
}

impl<'a> IntoIterator for NamedVectors<'a> {
    type Item = (CowKey<'a>, CowValue<'a>);

    type IntoIter =
        tinyvec::TinyVecIterator<[(CowKey<'a>, CowValue<'a>); super::tiny_map::CAPACITY]>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}
