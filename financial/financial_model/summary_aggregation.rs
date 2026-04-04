use super::{FinancialSummary, TimePoint};

fn aggregate_timeseries(all_series: &[Vec<TimePoint>]) -> Vec<TimePoint> {
    let max_len = all_series.iter().map(|s| s.len()).max().unwrap_or(0);
    let mut merged = Vec::with_capacity(max_len);

    for step_idx in 0..max_len {
        let mut avg_nw = 0.0_f32;
        let mut med_nw = 0.0_f32;
        let mut p10_nw = 0.0_f32;
        let mut p90_nw = 0.0_f32;
        let mut bankrupt = 0_u32;
        let mut liq = 0.0_f32;
        let mut k401 = 0.0_f32;
        let mut home_eq = 0.0_f32;
        let mut debt = 0.0_f32;
        let mut sample_count = 0_u32;
        let mut step = step_idx as u64;

        for series in all_series {
            if let Some(point) = series.get(step_idx) {
                step = point.step;
                avg_nw += point.average_net_worth;
                med_nw += point.median_net_worth;
                p10_nw += point.p10_net_worth;
                p90_nw += point.p90_net_worth;
                bankrupt += point.bankruptcy_count;
                liq += point.average_liquid_cash;
                k401 += point.average_401k;
                home_eq += point.average_home_equity;
                debt += point.average_debt;
                sample_count += 1;
            }
        }

        if sample_count > 0 {
            let sc = sample_count as f32;
            merged.push(TimePoint {
                step,
                average_net_worth: avg_nw / sc,
                median_net_worth: med_nw / sc,
                p10_net_worth: p10_nw / sc,
                p90_net_worth: p90_nw / sc,
                bankruptcy_count: (bankrupt as f32 / sc).round() as u32,
                average_liquid_cash: liq / sc,
                average_401k: k401 / sc,
                average_home_equity: home_eq / sc,
                average_debt: debt / sc,
            });
        }
    }

    merged
}

pub fn aggregate_summaries(mode: &str, summaries: &[FinancialSummary]) -> FinancialSummary {
    let first = summaries
        .first()
        .expect("Expected at least one summary for aggregation");
    let count = summaries.len() as f32;

    let mut avg_nw = 0.0_f32;
    let mut med_nw = 0.0_f32;
    let mut p10_nw = 0.0_f32;
    let mut p90_nw = 0.0_f32;
    let mut max_nw = f32::MIN;
    let mut min_nw = f32::MAX;
    let mut gini = 0.0_f32;
    let mut bankruptcies = 0_u32;
    let mut successful_retirements = 0_u32;
    let mut liq = 0.0_f32;
    let mut taxable = 0.0_f32;
    let mut k401 = 0.0_f32;
    let mut home_eq = 0.0_f32;
    let mut total_debt = 0.0_f32;
    let mut init_time = 0.0_f32;
    let mut step_time = 0.0_f32;
    let mut metrics_time = 0.0_f32;
    let mut run_duration = 0.0_f32;
    let mut overhead = 0.0_f32;
    let mut series = Vec::with_capacity(summaries.len());

    for s in summaries {
        avg_nw += s.average_net_worth;
        med_nw += s.median_net_worth;
        p10_nw += s.p10_net_worth;
        p90_nw += s.p90_net_worth;
        max_nw = max_nw.max(s.max_net_worth);
        min_nw = min_nw.min(s.min_net_worth);
        gini += s.gini_coefficient;
        bankruptcies += s.bankruptcy_count;
        successful_retirements += s.successful_retirement_count;
        liq += s.avg_liquid_cash;
        taxable += s.avg_taxable;
        k401 += s.avg_401k;
        home_eq += s.avg_home_equity;
        total_debt += s.avg_total_debt;
        init_time += s.init_time;
        step_time += s.step_compute_time;
        metrics_time += s.metrics_calc_time;
        run_duration += s.run_duration;
        overhead += s.communication_overhead;
        series.push(s.timeseries.clone());
    }

    FinancialSummary {
        mode: mode.to_string(),
        steps: first.steps,
        num_agents: first.num_agents,
        reps: first.reps,
        seed: first.seed,
        strategy_desc: first.strategy_desc.clone(),
        average_net_worth: avg_nw / count,
        median_net_worth: med_nw / count,
        p10_net_worth: p10_nw / count,
        p90_net_worth: p90_nw / count,
        max_net_worth: max_nw,
        min_net_worth: min_nw,
        gini_coefficient: (gini / count).clamp(0.0, 1.0),
        bankruptcy_count: (bankruptcies as f32 / count).round() as u32,
        successful_retirement_count: (successful_retirements as f32 / count).round() as u32,
        avg_liquid_cash: liq / count,
        avg_taxable: taxable / count,
        avg_401k: k401 / count,
        avg_home_equity: home_eq / count,
        avg_total_debt: total_debt / count,
        init_time: init_time / count,
        step_compute_time: step_time / count,
        metrics_calc_time: metrics_time / count,
        run_duration: run_duration / count,
        communication_overhead: overhead / count,
        timeseries: aggregate_timeseries(&series),
    }
}
