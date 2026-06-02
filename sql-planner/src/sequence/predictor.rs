use std::cmp::max;
use std::rc::Rc;

#[derive(Clone, Copy, Debug)]
pub struct Inertion {
    numerator: i64,
    denominator: i64,
}

impl std::fmt::Display for Inertion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

#[derive(Debug)]
pub enum InertionError {
    ZeroDenominator,
}

impl Inertion {
    pub fn new(num: i64, den: i64) -> Result<Self, InertionError> {
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

#[derive(Clone)]
pub struct Config {
    pub inertion: Inertion,
    pub action: Rc<dyn MakeAction>,
}

#[derive(Default, Clone, Copy)]
pub struct TermState {
    pub query_prediction_count: i64,
    pub query_actual_count: i64,

    pub query_pending_count: i64,
    pub id_remained_count: i64,
}

#[derive(Clone, Copy)]
pub struct Action {
    pub id_wildcard_recommended_count: i64,
}

#[derive(Clone, Copy)]
pub struct Prediction {
    pub query_count: i64,
}

impl<T> From<T> for Prediction
where
    T: Into<i64>,
{
    fn from(value: T) -> Self {
        Prediction {
            query_count: value.into(),
        }
    }
}

fn make_prediction(state: &PredictorState) -> Prediction {
    (state.stats.mean + state.stats.MAD).into()
}

pub trait MakeAction {
    fn make(&self, state: &PredictorState, pred: Prediction) -> Action;
}

fn div_up(a: i64, b: i64) -> i64 {
    (a + b - 1) / b
}

#[derive(Default, Clone)]
pub struct PredictorStats {
    pub mean: i64,
    pub MAD: i64,
    pub MSD: i64,
}

impl PredictorStats {
    pub fn clear(&mut self) {
        *self = <_>::default();
    }

    fn non_zero_values(term: &[TermState]) -> impl Iterator<Item = &TermState> {
        term.iter().filter(|x| x.query_actual_count > 0)
    }

    pub fn calculate(&mut self, term: &[TermState]) {
        let non_zero_len = {
            let len = Self::non_zero_values(term).count();
            if len > 0 {
                len
            } else {
                1
            }
        };

        let mean = div_up(
            Self::non_zero_values(term).map(|x| x.query_actual_count).sum(),
            non_zero_len as i64,
        );
        self.mean = mean;
        self.MAD = div_up(
            Self::non_zero_values(term)
                .map(|x| (x.query_actual_count as i64 - mean).abs())
                .sum(),
            non_zero_len as i64,
        );
        self.MSD = div_up(
            Self::non_zero_values(term)
                .map(|x| (x.query_actual_count as i64 - mean).pow(2))
                .sum(),
            non_zero_len as i64,
        );
    }
}

#[derive(Default, Clone)]
pub struct MeldingMax {
    value: i64,
}

impl MeldingMax {
    pub fn update(&mut self, value: i64) {
        self.value = max(self.value, value);
    }

    pub fn get(&self) -> i64 {
        self.value
    }

    pub fn meld(&mut self, inertion: &Inertion) {
        let x = self.value;

        self.value = x * inertion.numerator / inertion.denominator;
    }
}

#[derive(Default, Clone)]
pub struct PredictorState {
    pub term: [TermState; 10],

    pub stats: PredictorStats,
    pub max: MeldingMax,
}

impl PredictorState {
    fn push(&mut self, term: TermState, inertion: &Inertion) {
        self.stats.clear();
        self.max.meld(inertion);
        self.max.update(term.query_actual_count);

        self.term.rotate_left(1);
        *self.term.last_mut().unwrap() = term;
    }
}

pub struct Predictor {
    config: Config,
}

impl Predictor {
    pub fn new(config: Config) -> Self {
        Self { config: config }
    }

    /// Contract: Current Term is finished
    pub fn prepare_next_term(&self, state: &PredictorState) -> (Prediction, Action) {
        let prediction = make_prediction(state);
        let action = self.config.action.make(state, prediction);
        (prediction, action)
    }

    pub fn finish_term(&self, state: &mut PredictorState, term: TermState) {
        state.push(term, &self.config.inertion);
        state.stats.calculate(&state.term);
    }
}
