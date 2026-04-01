# Financial Simulation - Life Events Documentation

This document describes all the realistic life events modeled in the financial simulation engine.

## Overview

The simulation includes 10 major life events that can occur during an agent's lifetime, each with realistic probabilities and financial impacts based on real-world data.

## Life Events

### 1. Marriage
**When**: Ages 25-44  
**Probability**: 3% per year (configurable: `marriage_prob_per_year`)  
**Financial Impact**:
- Adds partner income ($30K-$110K range)
- Dual-income household for joint expenses
- Resets divorce timer

**Real-world basis**: Average marriage age in US is 28-30, with most marriages occurring before age 45.

---

### 2. Divorce
**When**: Ages 30+ (only if married, 10-year cooldown after previous divorce)  
**Probability**: 2.2% per year (configurable: `divorce_prob_per_year`)  
**Financial Impact**:
- Assets split 50/50 (cash, brokerage, 401k)
- Home equity buyout: mortgage increased by 50% of equity
- Partner income lost
- Base expenses increase by 25% (single household costs)
- Prevents remarriage for tracked period

**Real-world basis**: ~40-50% of marriages end in divorce, with peak risk in years 5-15. Average divorce rate translates to ~2-3% annual risk for married individuals.

---

### 3. Children
**When**: Ages 25-41 (only if married, max 4 children)  
**Probability**: 2.5% per year (configurable: `child_prob_per_year`)  
**Financial Impact**:
- Adds $17K annual expenses per child (configurable: `child_annual_cost`)
- Children age naturally and "leave" at age 18
- Expenses automatically adjust when children leave

**Real-world basis**: USDA estimates ~$17K/year to raise a child (middle-income family, 2015 dollars adjusted). Average family size 1.9 children.

---

### 4. Inheritance
**When**: Ages 35-74  
**Probability**: 0.8% per year (configurable: `inheritance_prob_per_year`)  
**Amount**: $50K-$300K (configurable: `inheritance_amount_range`)  
**Financial Impact**:
- 70% added to liquid cash
- 30% directly invested in taxable brokerage

**Real-world basis**: ~30% of Americans receive inheritances averaging $46K-$200K depending on wealth quintile. Peak inheritance age is 50-60.

---

### 5. Job Promotion
**When**: After 1+ years at current job, not retired/unemployed  
**Probability**: 15% per year base, declining with age (configurable: `promotion_prob_per_year`)  
**Raise**: 8%-25% (configurable: `promotion_raise_range`)  
**Financial Impact**:
- Base income increased by raise percentage
- Years at current job resets to 0

**Real-world basis**: BLS data shows ~12-15% of workers receive promotions annually, with higher rates early in career. Promotion raises average 10-15%.

---

### 6. Job Switch
**When**: After 2+ years at current job, not retired/unemployed  
**Probability**: 12% per year (configurable: `job_switch_prob_per_year`)  
**Income Change**: -5% to +35% (configurable: `job_switch_raise_range`)  
**Financial Impact**:
- Base income adjusted by change percentage
- Career growth rate randomized (new role/company trajectory)
- Years at current job resets to 0

**Real-world basis**: Average job tenure is 4.1 years in US. Job switchers see +10-15% raises on average, but some take lateral moves or pay cuts for better opportunities.

---

### 7. Location Move
**When**: Ages 25-64  
**Probability**: 8% per year (configurable: `location_move_prob_per_year`)  
**Cost of Living Change**: 0.70x to 1.40x (configurable: `location_expense_multiplier_range`)  
**Financial Impact**:
- Base expenses adjusted by new location multiplier
- Rent adjusted proportionally
- Moving costs: $2K-$10K deducted from liquid cash

**Real-world basis**: ~9-11% of Americans move annually (Census data). Cost of living varies dramatically (NYC/SF 1.4x national average, many Southern/Midwest cities 0.7-0.8x).

---

### 8. Unemployment
**When**: Any time while employed  
**Probability**: 4% per year (configurable: `job_loss_prob`)  
**Duration**: 9 months (0.75 years, configurable: `unemployment_duration_years`)  
**Financial Impact**:
- Income replaced by 40% unemployment benefits
- Years at current job resets to 0 when laid off
- Career growth paused during unemployment

**Real-world basis**: Pre-pandemic unemployment rate ~4%. Average unemployment duration 6-9 months. Unemployment benefits replace ~40-50% of income.

---

### 9. Medical Emergency
**When**: Any age  
**Probability**: 1.5% per year (configurable: `medical_emergency_prob`)  
**Cost**: $5K-$50K (configurable: `emergency_cost_range`)  
**Financial Impact**:
- One-time cost added to annual expenses
- Can push into credit card debt if insufficient liquid cash

**Real-world basis**: ~10-15% of Americans face unexpected medical costs >$1K annually. Catastrophic costs ($10K+) affect ~1-2% yearly.

---

### 10. Retirement
**When**: Based on strategy (Age 65 or FIRE goal achievement)  
**Probability**: Deterministic based on retirement goal  
**Financial Impact**:
- Base income replaced by 35% Social Security benefits
- No more 401k contributions
- Career progression stops

**Real-world basis**: Social Security replaces ~35-40% of pre-retirement income for median earners. Median retirement age is 65-67.

---

## Continuous Life Processes

### Career Growth
- **Normal raises**: Base salary grows at inflation*0.6 + career_growth_rate (0.5%-3.5% real growth)
- **Years tracking**: Increments annually to determine eligibility for promotions/switches
- **Partner income**: Grows at inflation*0.6 + 0-2% additional

### Children Aging
- Children age 1 year per step
- Automatically removed from household at age 18
- Expenses automatically adjust

### Inflation
- All expenses grow at inflation rate (2.5% default)
- Income growth includes inflation component

---

## Configuration

All life event probabilities and parameters are configurable in `config_comprehensive.json` under the `macro_economics` section:

```json
{
  "macro_economics": {
    "marriage_prob_per_year": 0.03,
    "divorce_prob_per_year": 0.022,
    "child_prob_per_year": 0.025,
    "child_annual_cost": 17000.0,
    "inheritance_prob_per_year": 0.008,
    "inheritance_amount_range": [50000.0, 300000.0],
    "promotion_prob_per_year": 0.15,
    "promotion_raise_range": [0.08, 0.25],
    "job_switch_prob_per_year": 0.12,
    "job_switch_raise_range": [-0.05, 0.35],
    "location_move_prob_per_year": 0.08,
    "location_expense_multiplier_range": [0.70, 1.40],
    "job_loss_prob": 0.04,
    "unemployment_duration_years": 0.75,
    "medical_emergency_prob": 0.015,
    "emergency_cost_range": [5000.0, 50000.0]
  }
}
```

## Event Interactions

Several events have complex interactions:

1. **Divorce + Marriage**: Cannot remarry for 10 years after divorce
2. **Children + Partner**: Can only have children while married
3. **Promotion + Job Switch**: Both reset years_at_current_job
4. **Unemployment + Job Loss**: Unemployment triggers job loss status
5. **Location Move + Housing**: Rent adjusts with location cost multiplier

## Testing Life Events

To verify life events are occurring realistically in your simulation:

1. Run simulation with high probability settings
2. Check agent snapshots at various ages
3. Verify income volatility matches real-world patterns
4. Confirm bankruptcy rates and retirement success rates are reasonable

## Future Enhancements

Potential additions to the life events system:

- **Disability**: Long-term income reduction with disability insurance
- **Major repairs**: Home maintenance emergencies (HVAC, roof, etc.)
- **Education expenses**: College tuition for children
- **Business ventures**: Self-employment or side hustles
- **Elder care**: Costs for aging parents
- **Downsizing**: Retirement home sale and rent/smaller home purchase
- **Part-time retirement**: Reduced working hours with bridge income
- **Pension benefits**: Additional retirement income streams
