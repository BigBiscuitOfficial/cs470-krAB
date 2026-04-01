# Comprehensive Financial Life Simulation

This project is a highly realistic, large-scale Agent-Based Model (ABM) designed to simulate human financial trajectories from early adulthood through retirement. Built on top of the **krABMaga** Rust framework, it serves as a "be your own researcher" tool to explore how macroeconomic factors, systemic inequalities, personal financial strategies, and random life shocks compound over a 45-year time horizon.

## 🧬 The Underlying Framework: `krABMaga`

[krABMaga](https://github.com/krABMaga-project/krABMaga) is a discrete-event, Agent-Based Modeling framework written in Rust. It is designed for absolute performance, memory safety, and massive scale. 

By leveraging krABMaga, this financial simulation can model hundreds of thousands of individual, heterogeneous agents simultaneously. Each agent holds its own state, makes decisions based on localized parameters, and steps through time (annually or monthly) to accrue wealth, incur debt, and experience life events.

### ⚡ Experimental Multithreading & MPI
One of the core strengths of building this on krABMaga is its architecture for high-performance computing:
* **Local Multithreading**: The framework inherently supports parallel execution. Agents can be partitioned and updated concurrently across available CPU cores, allowing for massive Monte Carlo sweeps of different financial strategies in seconds.
* **MPI (Message Passing Interface)**: For truly population-scale simulations (e.g., simulating the entire US population of 330 million), the framework supports MPI. This allows the simulation state to be distributed across a cluster of computing nodes, communicating boundaries and macroscopic state changes efficiently.

## 💰 The Financial Simulation

The simulation executes a rigorous 6-phase economic model to determine the terminal wealth, retirement success rate, and bankruptcy risk of its agents. It sweeps across dozens of user-defined strategy combinations (e.g., "Rent vs. Buy", "Aggressive vs. Minimum Debt Payoff", "100% Stocks vs. 60/40 Portfolio").

### Features by Phase

1. **Demographics & Inequality**
   - Agents are initialized with varying education levels, genders, and racial backgrounds, pulling from real-world probability distributions.
   - These traits deterministically impact their base income, unemployment risk, starting wealth, and cost of living.
2. **Debt & Credit Dynamics**
   - Agents can hold Student Loans, Auto Loans, and Credit Card debt.
   - The engine simulates minimum payments, interest accrual (e.g., 18% CC rates), and allows agents to execute "Avalanche" payoff strategies to avoid debt spirals.
3. **Major Life Events & Shocks**
   - Life is stochastic. Agents face annualized probabilities of marriage (combining assets and scaling income/expenses), divorce (halving net worth), childbirth, inheritance, long-term disability, and severe job loss.
4. **Housing & Real Estate**
   - Implements realistic constraints: agents cannot buy a home unless they save a 20% down payment.
   - Models amortized mortgages, property taxes, maintenance costs (1%), and historical housing appreciation. 
5. **Market Volatility & Retirement (Sequence of Returns Risk)**
   - Replaces flat returns with stochastic Box-Muller normal distributions for Inflation, Stocks, and Bonds.
   - Enforces strict decumulation rules in retirement (e.g., 4% safe withdrawal rate) using a "Bond Tent" strategy (selling bonds first during market crashes).
   - Features annual portfolio rebalancing.
6. **Complex Taxes & Estate Planning**
   - Replaces flat taxes with a progressive income tax bracket system.
   - Applies long-term capital gains taxes to taxable brokerage withdrawals and ordinary income tax to 401(k) drawdowns during retirement.

## 🚀 How to Run

The simulation is primarily driven via a flexible JSON configuration file (`examples/config_comprehensive.json`), which allows researchers to tweak demographic probabilities, tax brackets, and market volatility without recompiling the Rust core.

To run a serial Monte Carlo sweep of all financial strategies:

```bash
cargo run --release --example financial_serial examples/config_comprehensive.json
```

### Outputs
The simulation automatically generates:
* `summary.json`: High-level aggregated statistics (Median NW, Gini Coefficient, Retirement Success Rates).
* `sweep_results.csv`: Detailed performance metrics for every combination of housing, debt, and investment strategy.
* `report.html`: A visual summary of the run.

---
*Built as a complex architectural demonstration of Rust-based Agent-Based Modeling and macroeconomic simulation.*
