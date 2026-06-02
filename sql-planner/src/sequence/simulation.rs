use std::{collections::BTreeSet, iter::zip, mem::swap};

use rand::{self, random_range};

use super::{
    interpreter::{DebugState, Interpreter, Range},
    predictor::Config,
};

struct Sequence {
    id_free_first: Option<u64>,
}

impl Default for Sequence {
    fn default() -> Self {
        Self {
            id_free_first: Some(0),
        }
    }
}

impl Sequence {
    fn lock(&mut self, n: u64) -> Option<Range> {
        if n == 0 {
            return None;
        }

        let start = self.id_free_first?;
        let end = start.saturating_add(n);

        self.id_free_first = if end != u64::MAX { Some(end) } else { None };

        Some(Range::from_range(start, end))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Query {
    pub index: u64,

    pub term_start: u64,

    pub term_result: u64,
    pub id_result: u64,
}

pub struct Simulator {
    sequence: Sequence,
    queries: Vec<Query>,
    term_max: u64,
    drop_term_requests: BTreeSet<u64>,

    lock_requests_inflight: Vec<u64>,

    terms: Vec<Vec<DebugState>>,
    shards: Vec<Interpreter>,
}

fn fill_indices(q: &mut Vec<Query>) {
    for (index, query) in q.iter_mut().enumerate() {
        query.index = index as u64;
    }
}

pub fn generate_spikes(term_count: usize) -> Vec<Query> {
    let mut v = vec![0; term_count];
    v.fill_with(|| random_range(4..10));
    for _ in 0..(term_count / 10) {
        let index = random_range(0..v.len());
        v[index] += 50;
    }

    let mut queries = vec![];

    for (term, count) in v.into_iter().enumerate() {
        for _ in 0..count {
            queries.push(Query {
                index: 0,
                term_start: term as u64,
                term_result: 0,
                id_result: 0,
            });
        }
    }

    fill_indices(&mut queries);

    queries
}

pub fn generate_sparse(term_count: usize) -> Vec<Query> {
    let mut v = vec![0; term_count];
    v.fill_with(|| random_range(4..10));
    for _ in 0..(term_count / 10) {
        let index = random_range(0..v.len());
        v[index] += 50;
    }

    for (index, v) in v.iter_mut().enumerate() {
        if index % 4 != 0 {
            *v = 0;
        }
    }

    let mut queries = vec![];

    for (term, count) in v.into_iter().enumerate() {
        for _ in 0..count {
            queries.push(Query {
                index: 0,
                term_start: term as u64,
                term_result: 0,
                id_result: 0,
            });
        }
    }

    fill_indices(&mut queries);

    queries
}

pub fn generate_gambling(term_count: usize) -> Vec<Query> {
    let term_count = core::cmp::max(term_count, 10);

    let mut v = vec![0; term_count];
    v.fill_with(|| random_range(4..10));

    v[3] = 2000;

    let mut queries = vec![];

    for (term, count) in v.into_iter().enumerate() {
        for _ in 0..count {
            queries.push(Query {
                index: 0,
                term_start: term as u64,
                term_result: 0,
                id_result: 0,
            });
        }
    }

    fill_indices(&mut queries);

    queries
}

pub fn generate_drops(term_count: usize, drop_request_count: usize) -> BTreeSet<u64> {
    let mut drop_request: BTreeSet<u64> = <_>::default();
    for _ in 0..drop_request_count {
        let v = random_range(0..term_count);
        drop_request.insert(v as u64);
    }

    drop_request
}

impl Simulator {
    const SHARD_COUNT: usize = 5;
    const TERM_DELTA_MAX: u64 = 10;

    pub fn new(config: Config) -> Self {
        Self {
            sequence: <_>::default(),
            queries: <_>::default(),
            drop_term_requests: <_>::default(),
            lock_requests_inflight: <_>::default(),
            term_max: 0,
            terms: <_>::default(),
            shards: (0..Self::SHARD_COUNT)
                .map(|_| Interpreter::new(config.clone()))
                .collect(),
        }
    }

    pub fn set_queries(&mut self, queries: Vec<Query>) {
        self.term_max = queries.iter().map(|x| x.term_start).max().unwrap();
        self.queries = queries;
    }

    pub fn set_drop_term_request(&mut self, drop_term_request: BTreeSet<u64>) {
        self.drop_term_requests = drop_term_request;
    }

    fn advance_term(&mut self, _current_term: u64) {
        for shard in &mut self.shards {
            shard.finish_term();
        }

        let mut requests = self.create_lock_request();
        // swap(&mut requests, &mut self.lock_requests_inflight);

        self.terms.push(self.dump());

        let responses = requests
            .into_iter()
            .map(|n| self.sequence.lock(n))
            .collect();
        self.handle_lock_response(responses);

        for shard in &mut self.shards {
            shard.advance_term();
        }
    }

    fn create_lock_request(&self) -> Vec<u64> {
        self.shards
            .iter()
            .map(|s| s.create_lock_request())
            .collect()
    }

    fn handle_lock_response(&mut self, responses: Vec<Option<Range>>) {
        for (shard, range) in zip(self.shards.iter_mut(), responses.into_iter()) {
            if let Some(range) = range {
                shard.handle_lock_response(range);
            }
        }
    }

    fn dump(&self) -> Vec<DebugState> {
        self.shards.iter().map(|s| s.dump_state()).collect()
    }

    pub fn run(&mut self) {
        while self.queries.iter().find(|x| x.term_result == 0).is_some() {
            let current_term = self.shards.get(0).unwrap().current_term();
            if current_term > self.term_max + Self::TERM_DELTA_MAX {
                break;
            }

            for q in &mut self.queries {
                if q.term_result != 0 {
                    continue;
                }

                if q.term_start > current_term {
                    break;
                }

                let shard_index = q.index as usize % self.shards.len();
                let shard = self.shards.get_mut(shard_index).unwrap();

                if let Some(id) = shard.handle_query(q.term_start) {
                    q.id_result = id;
                    q.term_result = current_term;
                }
            }
            self.advance_term(current_term);
        }
    }

    pub fn extract_queries(&self) -> Vec<Query> {
        self.queries.clone()
    }

    pub fn extract_terms(&self) -> Vec<Vec<DebugState>> {
        self.terms.clone()
    }
}
