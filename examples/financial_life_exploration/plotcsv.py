import os
import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.ticker import ScalarFormatter
import numpy as np

# 1. Read the data
csv_filename = "scaling_results_detailed.csv"
try:
    df = pd.read_csv(csv_filename)
except FileNotFoundError:
    print(f"Error: Could not find '{csv_filename}'. Please ensure the file exists.")
    exit()

# Create output directory
output_dir = "plots"
os.makedirs(output_dir, exist_ok=True)

# 2. Derive additional insights
# Isolate pure compute time from MPI overhead
df['Corrected Compute Time (s)'] = df['Avg Total Max (s)'] * (1 - df['Avg Overhead Ratio'])

# Calculate True Weak Scaling Efficiency: T_base / T_N
# (Since problem size grows with ranks, ideal time is constant)
baseline_total = df['Avg Total Max (s)'].iloc[0]
df['Weak Scaling Efficiency (%)'] = (baseline_total / df['Avg Total Max (s)']) * 100

baseline_compute = df['Corrected Compute Time (s)'].iloc[0]

# Setup global style
plt.style.use('seaborn-v0_8-whitegrid')

def format_log_x():
    plt.xscale('log', base=2)
    plt.xticks(df['MPI Ranks'], df['MPI Ranks'])
    plt.xlabel('MPI Ranks', fontsize=12, fontweight='bold')

# =========================================================
# Chart 1: Raw Execution Time vs. Ideal (Log-Linear)
# Highlights the real-world performance degradation.
# =========================================================
plt.figure(figsize=(8, 5))
plt.plot(df['MPI Ranks'], df['Avg Total Max (s)'], marker='o', color='#d62728', linewidth=2, label='Actual Total Time')
plt.axhline(y=baseline_total, color='black', linestyle='--', linewidth=2, label='Ideal Weak Scaling')
format_log_x()
plt.ylabel('Time (s)', fontsize=12, fontweight='bold')
plt.title('Real-World Weak Scaling (Generation Time)', fontsize=14, fontweight='bold')
plt.legend()
plt.tight_layout()
plt.savefig(f'{output_dir}/01_raw_weak_scaling.png', dpi=300)
plt.close()

# =========================================================
# Chart 2: Corrected Compute Time vs. Ideal
# Proves that the underlying C++/math scales perfectly.
# =========================================================
plt.figure(figsize=(8, 5))
plt.plot(df['MPI Ranks'], df['Corrected Compute Time (s)'], marker='s', color='#1f77b4', linewidth=2, label='Compute Time (No Overhead)')
plt.axhline(y=baseline_compute, color='black', linestyle='--', linewidth=2, label='Ideal Compute Time')
format_log_x()
plt.ylabel('Time (s)', fontsize=12, fontweight='bold')
plt.title('Corrected Compute Time vs Ranks', fontsize=14, fontweight='bold')
plt.legend()
plt.tight_layout()
plt.savefig(f'{output_dir}/02_corrected_compute_time.png', dpi=300)
plt.close()

# =========================================================
# Chart 3: MPI Overhead Percentage (Area Chart)
# Visually emphasizes how much of the runtime is eaten by network.
# =========================================================
plt.figure(figsize=(8, 5))
overhead_percent = df['Avg Overhead Ratio'] * 100
plt.fill_between(df['MPI Ranks'], overhead_percent, color='#ff7f0e', alpha=0.5)
plt.plot(df['MPI Ranks'], overhead_percent, marker='^', color='#d62728', linewidth=2)
format_log_x()
plt.ylabel('% of Time Spent Communicating', fontsize=12, fontweight='bold')
plt.title('MPI Communication Overhead', fontsize=14, fontweight='bold')
plt.ylim(0, max(overhead_percent) + 10)
plt.tight_layout()
plt.savefig(f'{output_dir}/03_mpi_overhead_area.png', dpi=300)
plt.close()

# =========================================================
# Chart 4: True Weak Scaling Efficiency
# Shows how efficiency drops from 100% due to overhead.
# =========================================================
plt.figure(figsize=(8, 5))
plt.plot(df['MPI Ranks'], df['Weak Scaling Efficiency (%)'], marker='d', color='#2ca02c', linewidth=2)
plt.axhline(y=100, color='black', linestyle='--', linewidth=2, label='100% Efficiency')
format_log_x()
plt.ylabel('Efficiency (%)', fontsize=12, fontweight='bold')
plt.title('Weak Scaling Efficiency Drop-off', fontsize=14, fontweight='bold')
plt.ylim(0, 110)
plt.legend()
plt.tight_layout()
plt.savefig(f'{output_dir}/04_weak_scaling_efficiency.png', dpi=300)
plt.close()

# =========================================================
# Chart 5: Total Application Runtime
# Shows the holistic time cost to complete the simulation.
# =========================================================
plt.figure(figsize=(8, 5))
plt.plot(df['MPI Ranks'], df['Total Run Time (s)'], marker='X', color='#9467bd', linewidth=2)
format_log_x()
plt.ylabel('Total Run Time (s)', fontsize=12, fontweight='bold')
plt.title('Total Application Execution Time', fontsize=14, fontweight='bold')
plt.tight_layout()
plt.savefig(f'{output_dir}/05_total_runtime.png', dpi=300)
plt.close()

# =========================================================
# Chart 6: Algorithmic Stability (Fitness vs individuals)
# Verifies that spreading out the workload doesn't break the Genetic Algorithm
# =========================================================
plt.figure(figsize=(8, 5))
# For this one, X-axis is problem size, not ranks
plt.plot(df['individuals'], df['Best Fitness'], marker='o', color='#8c564b', linewidth=2)
plt.xscale('log', base=2)
plt.xticks(df['individuals'], df['individuals'], rotation=45)
plt.xlabel('Total Population (Individuals)', fontsize=12, fontweight='bold')
plt.ylabel('Best Fitness Score', fontsize=12, fontweight='bold')
plt.title('Solution Quality vs. Problem Size', fontsize=14, fontweight='bold')
# Adjust Y limits slightly to show the stable band clearly
min_fit = df['Best Fitness'].min()
max_fit = df['Best Fitness'].max()
plt.ylim(min_fit - 0.05, max_fit + 0.05)
plt.tight_layout()
plt.savefig(f'{output_dir}/06_fitness_stability.png', dpi=300)
plt.close()

print(f"Success! Generated 6 insightful charts in the '{output_dir}' directory.")
