use common::counter::hardware_counter::HardwareCounterCell;
use common::types::{PointOffsetType, ScoreType};
use sparse::common::sparse_vector::SparseVector;

use crate::vector_storage::query::{Query, TransformInto};
use crate::vector_storage::query_scorer::QueryScorer;
use crate::vector_storage::SparseVectorStorage;

pub struct SparseCustomQueryScorer<
    'a,
    TVectorStorage: SparseVectorStorage,
    TQuery: Query<SparseVector>,
> {
    vector_storage: &'a TVectorStorage,
    query: TQuery,
    hardware_counter: HardwareCounterCell,
}

impl<
        'a,
        TVectorStorage: SparseVectorStorage,
        TQuery: Query<SparseVector> + TransformInto<TQuery, SparseVector, SparseVector>,
    > SparseCustomQueryScorer<'a, TVectorStorage, TQuery>
{
    pub fn new(query: TQuery, vector_storage: &'a TVectorStorage) -> Self {
        let query: TQuery = TransformInto::transform(query, |mut vector| {
            vector.sort_by_indices();
            Ok(vector)
        })
        .unwrap();

        Self {
            vector_storage,
            query,
            hardware_counter: HardwareCounterCell::new(),
        }
    }
}

impl<'a, TVectorStorage: SparseVectorStorage, TQuery: Query<SparseVector>> QueryScorer<SparseVector>
    for SparseCustomQueryScorer<'a, TVectorStorage, TQuery>
{
    #[inline]
    fn score_stored(&self, idx: PointOffsetType) -> ScoreType {
        let stored = self
            .vector_storage
            .get_sparse(idx)
            .expect("Failed to get sparse vector");
        self.query.score_by(|example| {
            let cpu_units = example.indices.len() + stored.indices.len();
            self.hardware_counter.cpu_counter().incr_delta(cpu_units);
            stored.score(example).unwrap_or(0.0)
        })
    }

    fn score(&self, v: &SparseVector) -> ScoreType {
        self.query.score_by(|example| {
            let cpu_units = v.indices.len() + example.indices.len();
            self.hardware_counter.cpu_counter().incr_delta(cpu_units);
            example.score(v).unwrap_or(0.0)
        })
    }

    fn score_internal(&self, _point_a: PointOffsetType, _point_b: PointOffsetType) -> ScoreType {
        unimplemented!("Custom scorer can compare against multiple vectors, not just one")
    }

    fn hardware_counter(&self) -> HardwareCounterCell {
        self.hardware_counter.take()
    }

    fn set_hardware_counter_checked(&self, checked: bool) {
        self.hardware_counter.set_checked(checked);
    }
}
