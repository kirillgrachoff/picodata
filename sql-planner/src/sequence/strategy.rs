use super::predictor::{Action, MakeAction, Prediction, PredictorState};
use core::cmp::max;

#[derive(Default, Clone, Copy)]
pub struct PredictedPredictedSmartActual {}
impl MakeAction for PredictedPredictedSmartActual {
    fn make(&self, state: &PredictorState, pred: Prediction) -> Action {
        let query_count = state.term.last().unwrap().query_actual_count;
        let delta = max(pred.query_count, query_count);
        Action {
            id_wildcard_recommended_count: pred.query_count * 2 + delta,
        }
    }
}

/// Не подходит: не преодолевает потерю запросов
#[derive(Default, Clone, Copy)]
pub struct PredictedSmartActualSmartActual {}
impl MakeAction for PredictedSmartActualSmartActual {
    fn make(&self, state: &PredictorState, pred: Prediction) -> Action {
        let query_count = state.term.last().unwrap().query_actual_count;
        let delta = max(pred.query_count, query_count);
        Action {
            id_wildcard_recommended_count: pred.query_count + 2 * delta,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct OrRejected<M: MakeAction + Copy> {
    make_action: M,
}

impl<M: MakeAction + Copy> OrRejected<M> {
    pub fn new(make_action: M) -> Self {
        Self {
            make_action: make_action,
        }
    }
}

impl<M: MakeAction + Copy> MakeAction for OrRejected<M> {
    fn make(&self, state: &PredictorState, pred: Prediction) -> Action {
        let action = self.make_action.make(state, pred);
        if action.id_wildcard_recommended_count == 0 {
            Action {
                id_wildcard_recommended_count: state.term.last().unwrap().query_pending_count,
            }
        } else {
            action
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct RejectedPredictedSmartActualSmartActual {}
impl MakeAction for RejectedPredictedSmartActualSmartActual {
    fn make(&self, state: &PredictorState, pred: Prediction) -> Action {
        let term = state.term.last().unwrap();

        let query_count = term.query_actual_count;
        let delta = max(pred.query_count, query_count);
        Action {
            id_wildcard_recommended_count: term.query_pending_count + pred.query_count + 2 * delta,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct RejectedPredicted {}
impl MakeAction for RejectedPredicted {
    fn make(&self, state: &PredictorState, pred: Prediction) -> Action {
        let term = state.term.last().unwrap();
        Action {
            id_wildcard_recommended_count: term.query_pending_count + pred.query_count,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct RejectedPredictedSmartActual {}
impl MakeAction for RejectedPredictedSmartActual {
    fn make(&self, state: &PredictorState, pred: Prediction) -> Action {
        let term = state.term.last().unwrap();
        let query_count = term.query_actual_count;
        let delta = max(pred.query_count, query_count);
        Action {
            id_wildcard_recommended_count: term.query_pending_count + pred.query_count + delta,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct RejectedPredictedActual {}
impl MakeAction for RejectedPredictedActual {
    fn make(&self, state: &PredictorState, pred: Prediction) -> Action {
        let term = state.term.last().unwrap();
        Action {
            id_wildcard_recommended_count: term.query_pending_count
                + pred.query_count
                + term.query_actual_count,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct RejectedActual {}
impl MakeAction for RejectedActual {
    fn make(&self, state: &PredictorState, _pred: Prediction) -> Action {
        let term = state.term.last().unwrap();
        Action {
            id_wildcard_recommended_count: term.query_pending_count + term.query_actual_count,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Rejected {}
impl MakeAction for Rejected {
    fn make(&self, state: &PredictorState, _pred: Prediction) -> Action {
        let term = state.term.last().unwrap();
        Action {
            id_wildcard_recommended_count: term.query_pending_count,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct MeldMax3Predicted {}
impl MakeAction for MeldMax3Predicted {
    fn make(&self, state: &PredictorState, pred: Prediction) -> Action {
        let m = state.max.get();
        let p = 3 * pred.query_count;
        Action {
            id_wildcard_recommended_count: max(m, p),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct MeldMax3Sigma {}
impl MakeAction for MeldMax3Sigma {
    fn make(&self, state: &PredictorState, _pred: Prediction) -> Action {
        let rmse = (state.stats.MSD as f64).sqrt().ceil() as i64;
        let m = state.max.get();
        let p = state.stats.mean + 3 * rmse;
        Action {
            id_wildcard_recommended_count: max(m, p),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct MeldMax3MaxPredicted {}
impl MakeAction for MeldMax3MaxPredicted {
    fn make(&self, state: &PredictorState, _pred: Prediction) -> Action {
        let rmse = (state.stats.MSD as f64).sqrt().ceil() as i64;
        let m = state.max.get();
        let p = state.stats.mean + 3 * rmse;
        Action {
            id_wildcard_recommended_count: max(m, 3 * p),
        }
    }
}
