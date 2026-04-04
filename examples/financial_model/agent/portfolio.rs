use super::person::Person;
use crate::financial_model::state::FinancialState;
use crate::financial_model::strategies::DebtStrategy;
use crate::financial_model::types::WithdrawalBreakdown;
use crate::financial_model::utils::utils::{allocate_by_target, pay_balance, rebalance_account};

/// Manage debt payments using aggressive or minimum strategies
pub fn manage_debt(
    person: &mut Person,
    state: &FinancialState,
    available_cash: &mut f32,
    total_minimum_debt_service: f32,
) {
    // Accrue annual interest on debt balances
    person.student_loan_debt *= 1.0 + state.config.debt.student_loan_rate;
    person.auto_loan_debt *= 1.0 + state.config.debt.auto_loan_rate;
    person.credit_card_debt *= 1.0 + state.config.debt.credit_card_rate;

    // Pay minimums first
    let paid_student_min = pay_balance(&mut person.student_loan_debt, total_minimum_debt_service);
    let paid_auto_min = pay_balance(&mut person.auto_loan_debt, total_minimum_debt_service);
    let paid_cc_min = pay_balance(&mut person.credit_card_debt, total_minimum_debt_service);

    // If debt was paid off before consuming full budget, return remainder
    let unused_min_budget =
        total_minimum_debt_service - (paid_student_min + paid_auto_min + paid_cc_min);
    if unused_min_budget > 0.0 {
        *available_cash += unused_min_budget;
    }

    // Aggressive paydown: avalanche method (highest interest rate first)
    if *available_cash > 0.0 && person.strategy.debt_paydown == DebtStrategy::Aggressive {
        loop {
            let mut target_rate = f32::NEG_INFINITY;
            let mut target_slot: u8 = 255;

            if person.student_loan_debt > 0.0 && state.config.debt.student_loan_rate > target_rate {
                target_rate = state.config.debt.student_loan_rate;
                target_slot = 0;
            }
            if person.auto_loan_debt > 0.0 && state.config.debt.auto_loan_rate > target_rate {
                target_rate = state.config.debt.auto_loan_rate;
                target_slot = 1;
            }
            if person.credit_card_debt > 0.0 && state.config.debt.credit_card_rate > target_rate {
                target_slot = 2;
            }

            if target_slot == 255 {
                break;
            }

            let paid = match target_slot {
                0 => pay_balance(&mut person.student_loan_debt, *available_cash),
                1 => pay_balance(&mut person.auto_loan_debt, *available_cash),
                _ => pay_balance(&mut person.credit_card_debt, *available_cash),
            };

            if paid <= 0.0 {
                break;
            }
            *available_cash -= paid;
            if *available_cash <= 0.0 {
                break;
            }
        }
    }
}

/// Handle cash shortfalls: retirees withdraw from portfolios, workers take on CC debt
pub fn handle_cash_shortfall(
    person: &mut Person,
    state: &FinancialState,
    available_cash: &mut f32,
    taxable_income: f32,
    federal_tax: f32,
    actual_stock_return: f32,
) {
    if person.retired {
        handle_retirement_withdrawal(
            person,
            state,
            available_cash,
            taxable_income,
            federal_tax,
            actual_stock_return,
        );
    } else {
        // Take on credit card debt
        person.credit_card_debt += -(*available_cash);
        person.credit_card_debt = person
            .credit_card_debt
            .min(state.config.debt.credit_card_limit);
        *available_cash = 0.0;
    }
}

/// Handle retirement withdrawals with tax implications
fn handle_retirement_withdrawal(
    person: &mut Person,
    state: &FinancialState,
    available_cash: &mut f32,
    taxable_income: f32,
    federal_tax: f32,
    actual_stock_return: f32,
) {
    let spending_gap = -(*available_cash);
    let safe_withdrawal_rate = state
        .config
        .macro_economics
        .safe_withdrawal_rate
        .clamp(0.0, 1.0);
    let portfolio = person.portfolio_value().max(0.0);
    let safe_draw = portfolio * safe_withdrawal_rate;
    let desired_withdrawal = spending_gap.max(safe_draw);
    let prefer_bonds = actual_stock_return < 0.0;
    let withdrawals = withdraw_in_order(person, desired_withdrawal, prefer_bonds);
    *available_cash += withdrawals.total();

    // Tax on brokerage withdrawals (capital gains)
    let brokerage_withdrawals = withdrawals.brokerage_withdrawals();
    if brokerage_withdrawals > 0.0 {
        let taxable_gains = brokerage_withdrawals * 0.50;
        let capital_gains_tax = taxable_gains
            * state
                .config
                .tax_system
                .long_term_capital_gains_rate
                .clamp(0.0, 1.0);
        let paid_from_liquid_cash = person.liquid_cash.min(capital_gains_tax).max(0.0);
        person.liquid_cash -= paid_from_liquid_cash;
        let unpaid_tax = (capital_gains_tax - paid_from_liquid_cash).max(0.0);
        *available_cash -= unpaid_tax;
    }

    // Tax on 401k withdrawals (ordinary income)
    let k401_withdrawals = withdrawals.k401_withdrawals();
    if k401_withdrawals > 0.0 {
        let revised_income_tax = state.calculate_income_tax(taxable_income + k401_withdrawals);
        let additional_income_tax = (revised_income_tax - federal_tax).max(0.0);
        let paid_from_liquid_cash = person.liquid_cash.min(additional_income_tax).max(0.0);
        person.liquid_cash -= paid_from_liquid_cash;
        let unpaid_tax = (additional_income_tax - paid_from_liquid_cash).max(0.0);
        *available_cash -= unpaid_tax;
    }

    if *available_cash < 0.0 {
        person.liquid_cash = 0.0;
        *available_cash = 0.0;
    }
}

/// Withdraw funds in priority order: cash -> brokerage -> 401k
fn withdraw_in_order(
    person: &mut Person,
    mut amount_needed: f32,
    prefer_bonds: bool,
) -> WithdrawalBreakdown {
    if amount_needed <= 0.0 {
        return WithdrawalBreakdown::default();
    }

    let mut breakdown = WithdrawalBreakdown::default();

    // Withdraw from cash first
    let from_cash = person.liquid_cash.min(amount_needed).max(0.0);
    person.liquid_cash -= from_cash;
    amount_needed -= from_cash;
    breakdown.from_cash = from_cash;

    // Then brokerage, then 401k
    let (first_brokerage, second_brokerage, first_401k, second_401k) = if prefer_bonds {
        (
            &mut person.brokerage_bonds,
            &mut person.brokerage_stocks,
            &mut person.k401_bonds,
            &mut person.k401_stocks,
        )
    } else {
        (
            &mut person.brokerage_stocks,
            &mut person.brokerage_bonds,
            &mut person.k401_stocks,
            &mut person.k401_bonds,
        )
    };

    let from_first_brokerage = first_brokerage.min(amount_needed).max(0.0);
    *first_brokerage -= from_first_brokerage;
    amount_needed -= from_first_brokerage;

    let from_second_brokerage = second_brokerage.min(amount_needed).max(0.0);
    *second_brokerage -= from_second_brokerage;
    amount_needed -= from_second_brokerage;

    let from_first_401k = first_401k.min(amount_needed).max(0.0);
    *first_401k -= from_first_401k;
    amount_needed -= from_first_401k;

    let from_second_401k = second_401k.min(amount_needed).max(0.0);
    *second_401k -= from_second_401k;

    if prefer_bonds {
        breakdown.from_brokerage_bonds = from_first_brokerage;
        breakdown.from_brokerage_stocks = from_second_brokerage;
        breakdown.from_k401_bonds = from_first_401k;
        breakdown.from_k401_stocks = from_second_401k;
    } else {
        breakdown.from_brokerage_stocks = from_first_brokerage;
        breakdown.from_brokerage_bonds = from_second_brokerage;
        breakdown.from_k401_stocks = from_first_401k;
        breakdown.from_k401_bonds = from_second_401k;
    }

    breakdown
}

/// Invest surplus cash in brokerage accounts
pub fn invest_surplus(person: &mut Person, available_cash: &mut f32) {
    let emergency_buffer = person.annual_base_expenses * 0.5;

    // Build emergency fund first
    if person.liquid_cash < emergency_buffer {
        let to_buffer = (emergency_buffer - person.liquid_cash).min(*available_cash);
        person.liquid_cash += to_buffer;
        *available_cash -= to_buffer;
    }

    // Invest remaining surplus
    if *available_cash > 0.0 {
        let to_invest = *available_cash * 0.80;
        let target_stock_fraction = person.strategy.asset_allocation.stocks.clamp(0.0, 1.0);
        let (to_brokerage_stocks, to_brokerage_bonds) =
            allocate_by_target(to_invest, target_stock_fraction);
        person.brokerage_stocks += to_brokerage_stocks;
        person.brokerage_bonds += to_brokerage_bonds;
        person.liquid_cash += *available_cash - to_invest;
        *available_cash = 0.0;
    }
}

/// Apply market returns and rebalance portfolios
pub fn apply_market_returns(
    person: &mut Person,
    actual_stock_return: f32,
    actual_bond_return: f32,
) {
    // Apply returns
    person.brokerage_stocks *= 1.0 + actual_stock_return;
    person.brokerage_bonds *= 1.0 + actual_bond_return;
    person.k401_stocks *= 1.0 + actual_stock_return;
    person.k401_bonds *= 1.0 + actual_bond_return;

    // Rebalance only during accumulation phase
    if !person.retired {
        let target_stock_fraction = person.strategy.asset_allocation.stocks.clamp(0.0, 1.0);
        rebalance_account(
            &mut person.brokerage_stocks,
            &mut person.brokerage_bonds,
            target_stock_fraction,
        );
        rebalance_account(
            &mut person.k401_stocks,
            &mut person.k401_bonds,
            target_stock_fraction,
        );
    }
}

/// Clamp values to prevent simulation blow-up and handle invalid states
pub fn clamp_accounts(person: &mut Person, state: &FinancialState) {
    person.liquid_cash = person.liquid_cash.clamp(-50_000.0, 50_000_000.0);
    person.brokerage_stocks = person.brokerage_stocks.clamp(0.0, 100_000_000.0);
    person.brokerage_bonds = person.brokerage_bonds.clamp(0.0, 100_000_000.0);
    person.k401_stocks = person.k401_stocks.clamp(0.0, 100_000_000.0);
    person.k401_bonds = person.k401_bonds.clamp(0.0, 100_000_000.0);
    person.credit_card_debt = person
        .credit_card_debt
        .clamp(0.0, state.config.debt.credit_card_limit);

    // Handle infinite/NaN values
    if !person.net_worth().is_finite() {
        person.liquid_cash = 0.0;
        person.brokerage_stocks = 0.0;
        person.brokerage_bonds = 0.0;
        person.k401_stocks = 0.0;
        person.k401_bonds = 0.0;
    }
}
