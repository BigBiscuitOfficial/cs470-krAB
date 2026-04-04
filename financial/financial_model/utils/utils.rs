use rand::Rng;
use std::collections::HashMap;

/// Sample a value from a range, or fallback if invalid
pub fn sample_from_range_or(rng: &mut impl Rng, range: [f32; 2], fallback: [f32; 2]) -> f32 {
    let (min, max) = if range[1] > range[0] {
        (range[0], range[1])
    } else {
        (fallback[0], fallback[1])
    };
    if min >= max {
        min
    } else {
        rng.random_range(min..max)
    }
}

/// Get multiplier from map or return default
pub fn map_multiplier_or(map: &HashMap<String, f32>, key: &str, default: f32) -> f32 {
    map.get(key).copied().unwrap_or(default)
}

/// Calculate annual minimum payment for a debt
pub fn annual_min_payment(balance: f32, rate: f32, floor: f32) -> f32 {
    if balance <= 0.0 {
        0.0
    } else {
        (balance * rate).max(floor).min(balance)
    }
}

/// Pay down a balance with available funds
pub fn pay_balance(balance: &mut f32, payment_budget: f32) -> f32 {
    if *balance <= 0.0 || payment_budget <= 0.0 {
        0.0
    } else {
        let paid = payment_budget.min(*balance);
        *balance -= paid;
        paid
    }
}

/// Sample from a normal distribution using Box-Muller transform
pub fn sample_normal(rng: &mut impl Rng, mean: f32, std_dev: f32) -> f32 {
    if std_dev <= 0.0 {
        return mean;
    }

    let u1 = rng.random_range(f32::EPSILON..1.0);
    let u2 = rng.random_range(0.0..1.0);
    let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
    mean + std_dev * z0
}

/// Allocate an amount between stocks and bonds based on target fraction
pub fn allocate_by_target(amount: f32, target_stock_fraction: f32) -> (f32, f32) {
    let stock_fraction = target_stock_fraction.clamp(0.0, 1.0);
    let stock = amount.max(0.0) * stock_fraction;
    let bonds = amount.max(0.0) - stock;
    (stock, bonds)
}

/// Rebalance an account to match target stock/bond allocation
pub fn rebalance_account(stocks: &mut f32, bonds: &mut f32, target_stock_fraction: f32) {
    let total = (*stocks + *bonds).max(0.0);
    if total <= 0.0 {
        *stocks = 0.0;
        *bonds = 0.0;
        return;
    }

    let target_stock = total * target_stock_fraction.clamp(0.0, 1.0);
    *stocks = target_stock;
    *bonds = total - target_stock;
}
