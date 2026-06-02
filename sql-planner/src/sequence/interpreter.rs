use std::{collections::VecDeque, fmt::Debug};

use super::predictor::{Config, MakeAction, Predictor, TermState, PredictorState};

#[derive(Default)]
pub struct Range {
    begin: u64,
    end: u64,
}

impl Range {
    pub fn from_range(begin: u64, end: u64) -> Range {
        Range { begin, end }
    }

    pub fn from_c_range(start: u64, count: u64) -> Range {
        Range {
            begin: start,
            end: start + count,
        }
    }

    pub fn count(&self) -> u64 {
        if self.is_empty() {
            0
        } else {
            self.end - self.begin
        }
    }

    pub fn len(&self) -> usize {
        self.count() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.begin >= self.end
    }

    pub fn next(&mut self) -> Option<u64> {
        if self.is_empty() {
            None
        } else {
            let result = self.begin;
            self.begin += 1;

            Some(result)
        }
    }
}

#[derive(Default)]
struct RangeSet {
    id_free_list: VecDeque<Range>,
    count: i64,
}

impl RangeSet {
    fn add(&mut self, r: Range) {
        self.count += r.count() as i64;
        self.id_free_list.push_back(r);
    }

    fn count(&self) -> i64 {
        self.count
    }

    fn clear(&mut self) {
        self.id_free_list.clear();
        self.count = 0;
    }

    fn get(&mut self) -> Option<u64> {
        let front = self.id_free_list.front_mut()?;
        if let Some(v) = front.next() {
            self.count -= 1;
            Some(v)
        } else {
            self.id_free_list.pop_front();
            self.get()
        }
    }
}

pub struct Interpreter {
    predictor: Predictor,
    predictor_state: PredictorState,

    current_term_id: u64,

    wildcard_ids: RangeSet,

    current_term_query_count: i64,
    current_term_query_reject_count: i64,
    current_term_query_prediction_count: i64,
}

impl Interpreter {
    pub fn new(config: Config) -> Self {
        Self {
            predictor: Predictor::new(config),
            predictor_state: <_>::default(),
            current_term_id: 0,
            wildcard_ids: <_>::default(),
            current_term_query_reject_count: 0,
            current_term_query_count: 0,
            current_term_query_prediction_count: 0,
        }
    }

    pub fn current_term(&self) -> u64 {
        self.current_term_id
    }

    fn make_term_state(&self) -> TermState {
        TermState {
            query_prediction_count: self.current_term_query_prediction_count,
            query_actual_count: self.current_term_query_count,
            query_pending_count: self.current_term_query_reject_count,
            id_remained_count: self.wildcard_ids.count() as i64,
        }
    }

    pub fn handle_lock_response(&mut self, range: Range) {
        self.wildcard_ids.add(range);
    }

    pub fn handle_query(&mut self, term_id: u64) -> Option<u64> {
        if term_id == self.current_term_id {
            self.current_term_query_count += 1;
        }
        let result = self.wildcard_ids.get();
        self.current_term_query_reject_count += result.is_none() as i64;
        result
    }

    pub fn finish_term(&mut self) {
        let state = self.make_term_state();
        self.predictor.finish_term(&mut self.predictor_state, state);
    }

    pub fn create_lock_request(&self) -> u64 {
        let (_, action) = self.predictor.prepare_next_term(&self.predictor_state);
        let r_count = action.id_wildcard_recommended_count;

        if r_count == 0 {
            0
        } else if r_count < self.wildcard_ids.count() {
            0
        } else {
            (r_count - self.wildcard_ids.count()) as u64
        }
    }

    pub fn advance_term(&mut self) {
        let (prediction, action) = self.predictor.prepare_next_term(&self.predictor_state);

        self.current_term_id += 1;

        self.current_term_query_count = 0;
        self.current_term_query_prediction_count = prediction.query_count;
        self.current_term_query_reject_count = 0;

        if action.id_wildcard_recommended_count == 0 {
            self.wildcard_ids.clear();
        }
    }

    pub(crate) fn dump_state(&self) -> DebugState {
        let state = self.make_term_state();

        let mut predictor_state = self.predictor_state.clone();
        self.predictor.finish_term(&mut predictor_state, state);

        let (_, action) = self.predictor.prepare_next_term(&predictor_state);

        DebugState {
            term: self.current_term(),
            prediction: state.query_prediction_count,
            actual: self.current_term_query_count,
            rejected: state.query_pending_count,
            id_remained: state.id_remained_count,
            id_recommended: action.id_wildcard_recommended_count,
            id_request: self.create_lock_request(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DebugState {
    pub term: u64,
    pub prediction: i64,
    pub actual: i64,
    pub rejected: i64,
    pub id_remained: i64,
    pub id_recommended: i64,
    pub id_request: u64,
}

impl std::fmt::Display for DebugState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(&self, f)
    }
}
