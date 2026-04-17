use krabmaga::*;
use std::sync::OnceLock;

#[cfg(feature = "distributed_mpi")]
use {
    crate::model::state::FinanceLifeState,
    krabmaga::{engine::schedule::Schedule, engine::state::State, rand::Rng},
    std::cmp::Ordering::Equal,
};

mod model;

pub struct ScaleConfig {
    pub households: u32,
    pub horizon: u64,
    pub individuals: u32,
    pub max_generation: u32,
    pub repetitions: u32,
}

impl ScaleConfig {
    fn from_env() -> Self {
        Self {
            households: read_env_u32("FIN_HOUSEHOLDS", HOUSEHOLDS),
            horizon: read_env_u64("FIN_HORIZON", HORIZON),
            individuals: read_env_u32("FIN_INDIVIDUALS", INDIVIDUALS),
            max_generation: read_env_u32("FIN_MAX_GENERATION", MAX_GENERATION),
            repetitions: read_env_u32("FIN_REPETITIONS", REPETITIONS),
        }
    }
}

static SCALE_CONFIG: OnceLock<ScaleConfig> = OnceLock::new();

pub fn scale_config() -> &'static ScaleConfig {
    SCALE_CONFIG.get_or_init(ScaleConfig::from_env)
}

fn read_env_u32(name: &str, default: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn read_env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

pub const HOUSEHOLDS: u32 = 48;
pub const HORIZON: u64 = 60;
pub const INDIVIDUALS: u32 = 256;
pub const MAX_GENERATION: u32 = 150;
pub const REPETITIONS: u32 = 8;
pub const DESIRED_FITNESS: f32 = 0.0;
pub const GENE_COUNT: usize = 7;
pub const MUTATION_RATE: f32 = 0.35;

#[cfg(not(feature = "distributed_mpi"))]
fn main() {
    println!("Enable the distributed_mpi feature to run this example.");
}

#[cfg(feature = "distributed_mpi")]
fn main() {
    let config = scale_config();
    if UNIVERSE.world().rank() == 0 {
        println!(
            "Scale config: households={} horizon={} individuals={} max_generation={} repetitions={}",
            config.households,
            config.horizon,
            config.individuals,
            config.max_generation,
            config.repetitions,
        );
    }

    let result = explore_ga_distributed_mpi!(
        init_population,
        fitness,
        selection,
        mutation,
        crossover,
        cmp,
        FinanceLifeState,
        DESIRED_FITNESS,
        config.max_generation,
        config.horizon,
        config.repetitions,
    );

    if !result.is_empty() {
        let name = "finance_life_explore_result".to_string();
        let _ = write_csv(&name, &result);
    }

    // Synchronize
    let world = UNIVERSE.world();
    world.barrier();
}

#[cfg(feature = "distributed_mpi")]
fn fitness(computed_ind: &mut Vec<(FinanceLifeState, Schedule)>) -> f32 {
    if computed_ind.is_empty() {
        return f32::MAX / 4.0;
    }

    let mut total = 0.0;
    for (state, _) in computed_ind.iter() {
        total += state.fitness_penalty();
    }

    total / computed_ind.len() as f32
}

#[cfg(feature = "distributed_mpi")]
fn cmp(fitness1: &f32, fitness2: &f32) -> bool {
    *fitness1 < *fitness2
}

#[cfg(feature = "distributed_mpi")]
fn init_population() -> Vec<String> {
    let mut population = Vec::new();
    let mut rng = krabmaga::rand::rng();
    let config = scale_config();

    for _ in 0..config.individuals {
        let genes = [
            rng.random_range(0.10..=0.95),
            rng.random_range(0.05..=0.90),
            rng.random_range(0.05..=0.95),
            rng.random_range(0.00..=1.00),
            rng.random_range(0.00..=1.00),
            rng.random_range(0.00..=1.00),
            rng.random_range(0.00..=1.00),
        ];
        population.push(format_genome(&genes));
    }

    population
}

#[cfg(feature = "distributed_mpi")]
fn selection(population_fitness: &mut Vec<(String, f32)>) {
    population_fitness.sort_by(|s1, s2| s1.1.partial_cmp(&s2.1).unwrap_or(Equal));
}

#[cfg(feature = "distributed_mpi")]
fn crossover(population: &mut Vec<String>) {
    if population.is_empty() {
        panic!("Population len can't be 0");
    }

    let mut rng = krabmaga::rand::rng();
    let elite_count = ((population.len() as f32) * 0.2).ceil() as usize;
    let elite_count = elite_count.max(1).min(population.len());
    let parent_pool = (population.len() / 2)
        .max(elite_count)
        .max(2)
        .min(population.len());

    let config = scale_config();

    let mut children: Vec<String> = population[..elite_count].to_vec();
    while children.len() < config.individuals as usize {
        let idx_one = rng.random_range(0..parent_pool);
        let mut idx_two = rng.random_range(0..parent_pool);
        while idx_one == idx_two {
            idx_two = rng.random_range(0..parent_pool);
        }

        let parent_one = parse_genome(&population[idx_one]);
        let parent_two = parse_genome(&population[idx_two]);
        let mut genes = [0.0; GENE_COUNT];

        for gene in 0..GENE_COUNT {
            let alpha = rng.random_range(0.35..=0.65);
            let noise = rng.random_range(-0.03..=0.03);
            genes[gene] = (parent_one[gene] * alpha + parent_two[gene] * (1.0 - alpha) + noise)
                .clamp(0.0, 1.0);
        }

        children.push(format_genome(&genes));
    }

    *population = children;
}

#[cfg(feature = "distributed_mpi")]
fn mutation(individual: &mut String) {
    let mut rng = krabmaga::rand::rng();
    if !rng.random_bool(MUTATION_RATE as f64) {
        return;
    }

    let mut genes = parse_genome(individual);
    let idx = rng.random_range(0..GENE_COUNT);
    let delta = rng.random_range(-0.10..=0.10);
    genes[idx] = (genes[idx] + delta).clamp(0.0, 1.0);
    *individual = format_genome(&genes);
}

#[cfg(feature = "distributed_mpi")]
fn parse_genome(individual: &str) -> [f32; GENE_COUNT] {
    let parts: Vec<&str> = individual.split(';').collect();
    assert_eq!(parts.len(), GENE_COUNT, "Expected {} genes", GENE_COUNT);

    let mut genes = [0.0; GENE_COUNT];
    for (idx, part) in parts.iter().enumerate() {
        genes[idx] = part
            .parse::<f32>()
            .expect("Unable to parse genome parameter to f32")
            .clamp(0.0, 1.0);
    }

    genes
}

#[cfg(feature = "distributed_mpi")]
fn format_genome(genes: &[f32; GENE_COUNT]) -> String {
    genes
        .iter()
        .map(|gene| format!("{:.5}", gene))
        .collect::<Vec<String>>()
        .join(";")
}
