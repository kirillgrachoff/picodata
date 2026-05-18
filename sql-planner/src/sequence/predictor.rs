use std::cmp::{max};

#[derive(Clone, Copy, Debug)]
pub struct Inertion {
    numerator: u64,
    denominator: u64,
}

#[derive(Debug)]
pub enum InertionError {
    ZeroDenominator,
}

impl Inertion {
    pub fn new(num: u64, den: u64) -> Result<Self, InertionError> {
        if den == 0 {
            Err(InertionError::ZeroDenominator)
        } else {
            Ok(Self {
                numerator: num,
                denominator: den,
            })
        }
    }
}

#[derive(Clone, Copy)]
pub struct Config<M: MakeAction + Copy> {
    pub inertion: Inertion,
    pub action: M,
}

#[derive(Default, Clone, Copy)]
pub struct TermState {
    pub query_prediction_count: u64,
    pub query_actual_count: u64,

    pub query_pending_count: u64,
    pub id_remained_count: u64,
}

#[derive(Clone, Copy)]
pub struct Action {
    pub id_wildcard_recommended_count: u64,
}

#[derive(Clone, Copy)]
pub struct Prediction {
    pub query_count: u64,
}

impl<T> From<T> for Prediction
where
    T: Into<u64>,
{
    fn from(value: T) -> Self {
        Prediction {
            query_count: value.into(),
        }
    }
}

fn make_prediction<M: MakeAction + Copy>(
    config: Config<M>,
    prediction_old: u64,
    query_count: u64,
) -> Prediction {
    let prediction_old = if prediction_old == 0 {
        query_count
    } else {
        prediction_old
    };

    let mut result = prediction_old * config.inertion.numerator
        + query_count * (config.inertion.denominator - config.inertion.numerator);

    result /= config.inertion.denominator;

    result.into()
}

pub trait MakeAction {
    fn make(&self, state: TermState, pred: Prediction) -> Action;
}

#[derive(Default, Clone, Copy)]
pub struct PredictedPredictedSmartActual {}
impl MakeAction for PredictedPredictedSmartActual {
    fn make(&self, state: TermState, pred: Prediction) -> Action {
        let query_count = state.query_actual_count;
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
    fn make(&self, state: TermState, pred: Prediction) -> Action {
        let query_count = state.query_actual_count;
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
    fn make(&self, state: TermState, pred: Prediction) -> Action {
        let action = self.make_action.make(state, pred);
        if action.id_wildcard_recommended_count == 0 {
            Action {
                id_wildcard_recommended_count: state.query_pending_count,
            }
        } else {
            action
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct RejectedPredictedSmartActualSmartActual {}
impl MakeAction for RejectedPredictedSmartActualSmartActual {
    fn make(&self, state: TermState, pred: Prediction) -> Action {
        let query_count = state.query_actual_count;
        let delta = max(pred.query_count, query_count);
        Action {
            id_wildcard_recommended_count: state.query_pending_count + pred.query_count + 2 * delta,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct RejectedPredicted {}
impl MakeAction for RejectedPredicted {
    fn make(&self, state: TermState, pred: Prediction) -> Action {
        Action {
            id_wildcard_recommended_count: state.query_pending_count + pred.query_count,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct RejectedPredictedSmartActual {}
impl MakeAction for RejectedPredictedSmartActual {
    fn make(&self, state: TermState, pred: Prediction) -> Action {
        let query_count = state.query_actual_count;
        let delta = max(pred.query_count, query_count);
        Action {
            id_wildcard_recommended_count: state.query_pending_count + pred.query_count + delta,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct RejectedPredictedActual {}
impl MakeAction for RejectedPredictedActual {
    fn make(&self, state: TermState, pred: Prediction) -> Action {
        Action {
            id_wildcard_recommended_count: state.query_pending_count + pred.query_count + state.query_actual_count,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct RejectedActual {}
impl MakeAction for RejectedActual {
    fn make(&self, state: TermState, _pred: Prediction) -> Action {
        Action {
            id_wildcard_recommended_count: state.query_pending_count + state.query_actual_count,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Rejected {}
impl MakeAction for Rejected {
    fn make(&self, state: TermState, _pred: Prediction) -> Action {
        Action {
            id_wildcard_recommended_count: state.query_pending_count,
        }
    }
}

pub struct Predictor<M: MakeAction + Copy> {
    config: Config<M>,
}

impl<M: MakeAction + Copy> Predictor<M> {
    pub fn new(config: Config<M>) -> Self {
        Self {
            config: config,
        }
    }

    pub fn prepare_next_term(&self, state: TermState) -> (Prediction, Action) {
        let prediction = make_prediction(self.config, state.query_prediction_count, state.query_actual_count);
        let action = self.config.action.make(state, prediction);
        (prediction, action)
    }
}
