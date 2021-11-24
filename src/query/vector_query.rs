use common::BitSet;
use qdrant_segment::types::ScoreType;

use crate::{DocId, DocSet, Score, Searcher, SegmentReader, TERMINATED, schema::Field};

use super::{BitSetDocSet, ConstScorer, Query, Scorer, Weight};

/// VectorQuery, query to look for similar documents to this vector.
#[derive(Clone, Debug)]
pub struct VectorQuery {
    field: Field,
    vector: Vec<f32>
}

impl VectorQuery {
    /// Creates a new VectorQuery
    pub fn new(field: Field, vector: Vec<f32>) -> VectorQuery {
        VectorQuery {
            field,
            vector: vector.clone()
        }
    }
}

impl Query for VectorQuery {
    fn weight(
        &self,
        searcher: &Searcher,
        scoring_enabled: bool,
    ) -> crate::Result<Box<dyn super::Weight>> {

        let schema = searcher.schema();
        // Check dimension of the vector.
        
        Ok(Box::new(VectorWeight {
            field: self.field,
            vector: self.vector.clone()
        }))
    }
}

pub struct VectorWeight {
    field: Field,
    vector: Vec<f32>
}

impl Weight for VectorWeight {
    fn scorer(
        &self,
        reader: &SegmentReader,
        boost: Score,
    ) -> crate::Result<Box<dyn super::Scorer>> {

        let reader = reader.vector_reader(self.field)?;
        // TODO: Pass limit.
        let docs = reader.search(&self.vector, 50);

        Ok(Box::new(VectorScorer::from_docs(docs)))
    }

    fn explain(
        &self,
        _reader: &crate::SegmentReader,
        _doc: crate::DocId,
    ) -> crate::Result<super::Explanation> {


        todo!()
    }
}

pub struct VectorScorer {
    docs: Vec<(DocId, ScoreType)>,
    i: usize
}

impl VectorScorer {
    pub fn from_docs(docs: Vec<(DocId, ScoreType)>) -> VectorScorer {
        VectorScorer {
            docs,
            i: 0
        }
    }
}

impl Scorer for VectorScorer {
    fn score(&mut self) -> Score {
        self.docs[self.i].1
    }
}

impl DocSet for VectorScorer {
    fn advance(&mut self) -> crate::DocId {
        if self.i < self.docs.len() {
            self.i += 1;
        } else {
            self.i = TERMINATED as usize;
        }
        self.i as u32
    }

    fn doc(&self) -> crate::DocId {
        self.docs[self.i].0
    }

    fn size_hint(&self) -> u32 {
        self.docs.len() as u32
    }
}