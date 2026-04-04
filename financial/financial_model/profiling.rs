use chrono::Utc;
use std::fs;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingEvent {
    SweepTotal,
    StrategyTotal,
    Init,
    StepCompute,
    MetricsCalc,
    RunDuration,
    CommunicationOverhead,
}

impl TimingEvent {
    pub fn as_str(self) -> &'static str {
        match self {
            TimingEvent::SweepTotal => "sweep_total",
            TimingEvent::StrategyTotal => "strategy_total",
            TimingEvent::Init => "init",
            TimingEvent::StepCompute => "step_compute",
            TimingEvent::MetricsCalc => "metrics_calc",
            TimingEvent::RunDuration => "run_duration",
            TimingEvent::CommunicationOverhead => "communication_overhead",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProfileContext {
    pub run_id: String,
    pub mode: String,
    pub num_agents: u32,
    pub num_steps: u32,
    pub num_reps: u32,
    pub num_threads: usize,
    pub num_ranks: usize,
    pub hostname: String,
    pub timestamp: String,
    pub seed: Option<u64>,
}

impl ProfileContext {
    pub fn new(
        mode: &str,
        num_agents: u32,
        num_steps: u32,
        num_reps: u32,
        num_threads: usize,
        seed: Option<u64>,
    ) -> Self {
        let timestamp = Utc::now().to_rfc3339();
        let run_id = format!("{}_{}", mode, Utc::now().format("%Y%m%d_%H%M%S_%3f"));
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string());

        Self {
            run_id,
            mode: mode.to_string(),
            num_agents,
            num_steps,
            num_reps,
            num_threads: num_threads.max(1),
            num_ranks: 1,
            hostname,
            timestamp,
            seed,
        }
    }

    pub fn total_cores(&self) -> usize {
        self.num_threads.max(1) * self.num_ranks.max(1)
    }
}

#[derive(Debug, Clone)]
pub struct TimingRecord {
    pub event: TimingEvent,
    pub strategy_index: Option<usize>,
    pub strategy_desc: String,
    pub init_time_s: f32,
    pub step_compute_s: f32,
    pub comm_overhead_s: f32,
    pub metrics_calc_s: f32,
    pub total_runtime_s: f32,
}

#[derive(Debug, Default)]
pub struct PerformanceProfile {
    records: Vec<TimingRecord>,
}

impl PerformanceProfile {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn record(
        &mut self,
        event: TimingEvent,
        strategy_index: Option<usize>,
        strategy_desc: &str,
        init_time_s: f32,
        step_compute_s: f32,
        comm_overhead_s: f32,
        metrics_calc_s: f32,
        total_runtime_s: f32,
    ) {
        self.records.push(TimingRecord {
            event,
            strategy_index,
            strategy_desc: strategy_desc.to_string(),
            init_time_s,
            step_compute_s,
            comm_overhead_s,
            metrics_calc_s,
            total_runtime_s,
        });
    }

    pub fn merge_from(&mut self, mut other: PerformanceProfile) {
        self.records.append(&mut other.records);
    }

    pub fn to_csv(&self, context: &ProfileContext) -> String {
        let mut out = String::from(
            "run_id,timestamp,mode,num_threads,num_ranks,total_cores,seed,strategy_id,strategy_index,strategy_desc,event,num_agents,num_steps,num_reps,init_time_s,step_compute_s,comm_overhead_s,metrics_calc_s,total_runtime_s,hostname\n",
        );

        for record in &self.records {
            let strategy_index = record
                .strategy_index
                .map(|v| v.to_string())
                .unwrap_or_default();
            let strategy_id = record
                .strategy_index
                .map(|v| format!("strategy_{}", v))
                .unwrap_or_else(|| "all_strategies".to_string());
            let seed = context.seed.map(|v| v.to_string()).unwrap_or_default();
            let escaped_desc = record.strategy_desc.replace('"', "\"\"");

            out.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},\"{}\",{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{}\n",
                context.run_id,
                context.timestamp,
                context.mode,
                context.num_threads,
                context.num_ranks,
                context.total_cores(),
                seed,
                strategy_id,
                strategy_index,
                escaped_desc,
                record.event.as_str(),
                context.num_agents,
                context.num_steps,
                context.num_reps,
                record.init_time_s,
                record.step_compute_s,
                record.comm_overhead_s,
                record.metrics_calc_s,
                record.total_runtime_s,
                context.hostname
            ));
        }

        out
    }

    pub fn export_csv(&self, context: &ProfileContext, output_path: &str) -> io::Result<()> {
        fs::write(output_path, self.to_csv(context))
    }

    pub fn export_csv_in_dir(
        &self,
        base_dir: &str,
        context: &ProfileContext,
    ) -> io::Result<String> {
        fs::create_dir_all(base_dir)?;
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S_%3f").to_string();
        let file_path = format!(
            "{}/profiling_{}_{}agents_{}steps_{}cores_{}.csv",
            base_dir,
            context.mode,
            context.num_agents,
            context.num_steps,
            context.total_cores(),
            timestamp
        );
        self.export_csv(context, &file_path)?;
        Ok(file_path)
    }
}
