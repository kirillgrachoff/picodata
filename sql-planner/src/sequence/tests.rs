use std::collections::BTreeSet;

use crate::sequence::{
    predictor::{
        Config, Inertion, MakeAction, OrRejected, PredictedPredictedSmartActual, PredictedSmartActualSmartActual, Rejected, RejectedActual, RejectedPredicted, RejectedPredictedActual, RejectedPredictedSmartActual, RejectedPredictedSmartActualSmartActual
    },
    simulation::Simulator,
};

#[test]
fn test_example() {
    let term_count = 50;

    let spikes = super::simulation::generate_spikes(term_count);
    let gambling = super::simulation::generate_gambling(term_count);
    let sparse = super::simulation::generate_sparse(term_count);
    let dropped = super::simulation::generate_drops(term_count, 10);
    run_dataset("spikes", &spikes, &dropped);
    run_dataset("gambling", &gambling, &dropped);
    run_dataset("sparse", &sparse, &dropped);
}

fn run_dataset(name: &str, queries: &Vec<super::simulation::Query>, dropped: &BTreeSet<u64>) {
    println!(
        "-- name: {} count: {} drops: {}",
        name,
        queries.len(),
        dropped.len()
    );

    run_strategy(Rejected {}, &queries, &dropped);

    run_strategy(RejectedActual {}, &queries, &dropped);
    run_strategy(RejectedPredicted {}, &queries, &dropped);
    run_strategy(RejectedPredictedActual {}, &queries, &dropped);
    run_strategy(RejectedPredictedSmartActual {}, &queries, &dropped);
    run_strategy(OrRejected::new(PredictedPredictedSmartActual {}), &queries, &dropped);
    run_strategy(OrRejected::new(PredictedSmartActualSmartActual {}), &queries, &dropped);
    run_strategy(
        RejectedPredictedSmartActualSmartActual {},
        &queries,
        &dropped,
    );

    println!("");
}

fn run_strategy<M: MakeAction + Copy>(
    action: M,
    queries: &Vec<super::simulation::Query>,
    dropped: &BTreeSet<u64>,
) {
    println!("");

    let inertion = vec![(3, 4), (1, 2), (1, 4), (1, 8)];
    for (num, den) in inertion {
        let config = Config {
            inertion: Inertion::new(num, den).unwrap(),
            action: action,
        };
        run_test(config, &queries, &dropped);
    }
}

fn run_test<M: MakeAction + Copy>(
    config: Config<M>,
    queries: &Vec<super::simulation::Query>,
    dropped: &BTreeSet<u64>,
) {
    let mut simulator = Simulator::new(config);
    simulator.set_queries(queries.clone());
    simulator.set_drop_term_request(dropped.clone());
    simulator.run();

    println!(
        "-- strategy: {}, inertion: {:?}",
        std::any::type_name::<M>(),
        config.inertion,
    );

    let stats_term = simulator.extract_terms();
    let stats_query = simulator.extract_queries();

    let mut query_rejected_count = 0_u64;
    let mut non_zero_request_count = 0_u64;
    let mut prediction_negative_diff = 0;

    for term in &stats_term {
        query_rejected_count += term
            .iter()
            .map(|x| x.rejected)
            .reduce(|a, b| a + b)
            .unwrap();
        non_zero_request_count += term
            .iter()
            .map(|x| (x.id_request > 0) as u64)
            .reduce(|a, b| a + b)
            .unwrap();
        prediction_negative_diff += term
            .iter()
            .filter(|x| x.prediction < x.actual)
            .map(|x| x.actual - x.prediction)
            .reduce(|a, b| a + b)
            .unwrap_or(0);
    }

    println!(
        "non_zero_requests: {} rejected: {} prediction_negative_diff: {}",
        non_zero_request_count, query_rejected_count, prediction_negative_diff,
    );

    let mut query_latency = 0;
    let mut non_completed = 0;
    let mut delayed = 0;
    let mut invertions = 0_u64;
    let mut id_latency = 0;
    let dropped_count = stats_query.iter().map(|x| x.id_result).max().unwrap() + 1 - stats_query.len() as u64;

    for query in &stats_query {
        query_latency += query.term_result.saturating_sub(query.term_start);
        non_completed += (query.term_result == 0) as u64;
        delayed += (query.term_start != query.term_result) as u64;
        id_latency += query.id_result.saturating_sub(query.index);
    }

    let mut seen_ids = vec![];
    for query in &stats_query {
        seen_ids.push(query.id_result);
        let mut more_than_current = 0;
        for id in &seen_ids {
            if query.id_result < *id {
                more_than_current += 1;
            }
        }
        invertions += more_than_current;
    }

    println!(
        "query_delayed_count: {} query_latency: {} invertions: {} dropped: {}",
        delayed,
        query_latency as f64 / stats_query.len() as f64,
        invertions,
        dropped_count,
    );
}
