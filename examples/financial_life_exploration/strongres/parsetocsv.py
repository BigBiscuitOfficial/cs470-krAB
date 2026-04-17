import os
import re
import csv
import glob

# Configuration
log_pattern = "scaling_run_*_procs.log"
csv_filename = "scaling_results_detailed.csv"

# Dictionary to hold the parsed data
data = {}
log_files = glob.glob(log_pattern)

for filename in log_files:
    match = re.search(r"scaling_run_(\d+)_procs\.log", filename)
    if not match:
        continue
        
    rank = int(match.group(1))
    
    # Variables to track
    config = {}
    best_fitness = "N/A"
    total_time = "N/A"
    
    total_max_sum = 0.0
    overhead_sum = 0.0
    count = 0
    
    with open(filename, 'r') as f:
        for line in f:
            # 1. Grab Input Sizes / Config
            if line.startswith("Scale config:"):
                # Dynamically extract all word=number pairs
                pairs = re.findall(r"(\w+)=([\d\.]+)", line)
                config = {k: float(v) if '.' in v else int(v) for k, v in pairs}

            # 2. Grab MPI Timing Averages
            elif line.startswith("[MPI_TIMING]"):
                total_max_match = re.search(r"total_max=([0-9\.]+)s", line)
                overhead_match = re.search(r"overhead_ratio=([0-9\.]+)", line)
                if total_max_match and overhead_match:
                    total_max_sum += float(total_max_match.group(1))
                    overhead_sum += float(overhead_match.group(1))
                    count += 1
            
            # 3. Grab Total Execution Time (Updates until the final generation)
            elif "Completed generation" in line:
                time_match = re.search(r"after ([0-9\.]+) seconds", line)
                if time_match:
                    total_time = float(time_match.group(1))
                    
            # 4. Grab Final Fitness Score
            elif "- Overall best fitness is" in line:
                fitness_match = re.search(r"is ([0-9\.]+)", line)
                if fitness_match:
                    best_fitness = float(fitness_match.group(1))

    # Only save if we actually processed MPI timings
    if count > 0:
        data[rank] = {
            'avg_total': total_max_sum / count,
            'avg_overhead': overhead_sum / count,
            'total_time': total_time,
            'best_fitness': best_fitness,
            **config # Unpack all the config inputs directly into our data dict
        }

# Sort data by rank
sorted_ranks = sorted(data.keys())

if not sorted_ranks:
    print(f"No valid log files found matching '{log_pattern}'.")
    exit()

base_rank = sorted_ranks[0]
base_time = data[base_rank]['avg_total']

# Dynamically build CSV header based on the config keys we found
base_data = data[base_rank]
config_keys = [k for k in base_data.keys() if k not in ['avg_total', 'avg_overhead', 'total_time', 'best_fitness']]

headers = [
    'MPI Ranks', 'Avg Total Max (s)', 'Speedup', 'Efficiency', 
    'Avg Overhead Ratio', 'Total Run Time (s)', 'Best Fitness'
] + config_keys # Appends households, horizon, individuals, etc.

# Write the fat CSV
with open(csv_filename, mode='w', newline='') as csv_file:
    writer = csv.writer(csv_file)
    writer.writerow(headers)

    print(f"Writing {len(headers)} columns to {csv_filename}...")

    for rank in sorted_ranks:
        d = data[rank]
        
        # Scaling Calculations
        speedup = base_time / d['avg_total']
        efficiency = speedup / (rank / base_rank)
        
        # Build the core row
        row = [
            rank, 
            round(d['avg_total'], 6), 
            round(speedup, 3), 
            round(efficiency, 3), 
            round(d['avg_overhead'], 3),
            d.get('total_time', 'N/A'),
            d.get('best_fitness', 'N/A')
        ]
        
        # Append the dynamic config sizes
        for k in config_keys:
            row.append(d.get(k, 'N/A'))
            
        writer.writerow(row)

print("--- Done!---")
