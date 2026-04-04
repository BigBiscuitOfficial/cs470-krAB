use serde_json;

use super::FinancialSummary;
use super::StrategyRunSummary;

/// Builds the interactive HTML report content used by sweep and single-run outputs.
///
/// Kept as a dedicated module to reduce cognitive load in `report.rs` while preserving
/// the exact output format and behavior.
pub fn generate_interactive_html(
    advice: &str,
    summary: Option<&FinancialSummary>,
    runs: &[StrategyRunSummary],
) -> String {
    let summary_json = summary
        .map(|s| serde_json::to_string(s).unwrap_or_else(|_| "{}".to_string()))
        .unwrap_or_else(|| "null".to_string());

    let runs_json = serde_json::to_string(runs).unwrap_or_else(|_| "[]".to_string());

    let has_timeseries = summary.map(|s| !s.timeseries.is_empty()).unwrap_or(false);

    // Build JavaScript section conditionally to match available data.
    let mut js_code = String::new();

    if has_timeseries {
        js_code.push_str(r#"
        // Funnel Chart
        if (summaryData && summaryData.timeseries) {
            const labels = summaryData.timeseries.map(p => `Year ${p.step}`);
            const ctx = document.getElementById('funnelChart').getContext('2d');
            new Chart(ctx, {
                type: 'line',
                data: {
                    labels: labels,
                    datasets: [
                        {
                            label: '90th Percentile (Best Case)',
                            data: summaryData.timeseries.map(p => p.p90_net_worth),
                            borderColor: 'rgba(52, 211, 153, 1)',
                            backgroundColor: 'rgba(52, 211, 153, 0.1)',
                            fill: '+1',
                            tension: 0.4
                        },
                        {
                            label: 'Median',
                            data: summaryData.timeseries.map(p => p.median_net_worth),
                            borderColor: 'rgba(102, 126, 234, 1)',
                            backgroundColor: 'rgba(102, 126, 234, 0.2)',
                            borderWidth: 3,
                            fill: '+1',
                            tension: 0.4
                        },
                        {
                            label: '10th Percentile (Worst Case)',
                            data: summaryData.timeseries.map(p => p.p10_net_worth),
                            borderColor: 'rgba(239, 68, 68, 1)',
                            backgroundColor: 'rgba(239, 68, 68, 0.1)',
                            fill: false,
                            tension: 0.4
                        }
                    ]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: {
                        legend: { display: true, position: 'top' },
                        tooltip: {
                            callbacks: {
                                label: function(context) {
                                    return context.dataset.label + ': $' + context.parsed.y.toLocaleString();
                                }
                            }
                        }
                    },
                    scales: {
                        y: {
                            ticks: {
                                callback: function(value) {
                                    return '$' + (value/1000).toFixed(0) + 'K';
                                }
                            }
                        }
                    }
                }
            });
        }
        "#);
    }

    if summary.is_some() {
        js_code.push_str(r#"
        // Metrics Grid
        if (summaryData) {
            const grid = document.getElementById('metricsGrid');
            const metrics = [
                { label: 'Median Net Worth', value: '$' + summaryData.median_net_worth.toLocaleString() },
                { label: 'Retirement Success', value: (summaryData.successful_retirement_count / summaryData.num_agents * 100).toFixed(1) + '%' },
                { label: 'Bankruptcy Rate', value: (summaryData.bankruptcy_count / summaryData.num_agents * 100).toFixed(1) + '%' },
                { label: 'Avg 401(k)', value: '$' + summaryData.avg_401k.toLocaleString() }
            ];
            metrics.forEach(m => {
                const card = document.createElement('div');
                card.className = 'metric-card';
                card.innerHTML = `<div class="metric-label">${m.label}</div><div class="metric-value">${m.value}</div>`;
                grid.appendChild(card);
            });
        }
        "#);
    }

    if runs.len() > 1 {
        js_code.push_str(r#"
        // Strategy Scatter
        if (runsData.length > 0) {
            const ctx2 = document.getElementById('strategyScatter').getContext('2d');
            new Chart(ctx2, {
                type: 'scatter',
                data: {
                    datasets: [{
                        label: 'Strategies',
                        data: runsData.map(r => ({
                            x: r.bankruptcy_rate * 100,
                            y: r.median_net_worth / 1000,
                            label: r.strategy_desc
                        })),
                        backgroundColor: 'rgba(102, 126, 234, 0.6)',
                        borderColor: 'rgba(102, 126, 234, 1)',
                        pointRadius: 8,
                        pointHoverRadius: 12
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: {
                        legend: { display: false },
                        tooltip: {
                            callbacks: {
                                label: function(context) {
                                    return context.raw.label + ': $' + (context.raw.y * 1000).toLocaleString() + ', ' + context.raw.x.toFixed(1) + '% bankruptcy';
                                }
                            }
                        }
                    },
                    scales: {
                        x: {
                            title: { display: true, text: 'Bankruptcy Rate (%)' }
                        },
                        y: {
                            title: { display: true, text: 'Median Net Worth ($K)' }
                        }
                    }
                }
            });

            // Strategy Table
            const tbody = document.getElementById('strategyTableBody');
            runsData.sort((a, b) => {
                const scoreA = a.median_net_worth - a.bankruptcy_rate * 500000 + a.successful_retirement_rate * 200000;
                const scoreB = b.median_net_worth - b.bankruptcy_rate * 500000 + b.successful_retirement_rate * 200000;
                return scoreB - scoreA;
            }).slice(0, 10).forEach(run => {
                const score = run.median_net_worth - run.bankruptcy_rate * 500000 + run.successful_retirement_rate * 200000;
                const bankruptcyBadge = run.bankruptcy_rate < 0.05 ? 'badge-success' : run.bankruptcy_rate < 0.15 ? 'badge-warning' : 'badge-danger';
                const row = document.createElement('tr');
                row.innerHTML = `
                    <td>${run.strategy_desc}</td>
                    <td>$${run.median_net_worth.toLocaleString()}</td>
                    <td><span class="badge badge-success">${(run.successful_retirement_rate * 100).toFixed(1)}%</span></td>
                    <td><span class="badge ${bankruptcyBadge}">${(run.bankruptcy_rate * 100).toFixed(1)}%</span></td>
                    <td>${score.toLocaleString()}</td>
                `;
                tbody.appendChild(row);
            });
        }
        "#);
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Financial Life Simulation - Comprehensive Report</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.min.js"></script>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #1a202c;
            padding: 2rem;
            min-height: 100vh;
        }}
        .container {{
            max-width: 1400px;
            margin: 0 auto;
        }}
        .header {{
            background: white;
            border-radius: 16px;
            padding: 2rem;
            margin-bottom: 2rem;
            box-shadow: 0 10px 40px rgba(0,0,0,0.1);
        }}
        .header h1 {{
            font-size: 2.5rem;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
            margin-bottom: 0.5rem;
        }}
        .header .subtitle {{
            color: #718096;
            font-size: 1.1rem;
        }}
        .card {{
            background: white;
            border-radius: 16px;
            padding: 2rem;
            margin-bottom: 2rem;
            box-shadow: 0 4px 20px rgba(0,0,0,0.08);
        }}
        .card h2 {{
            font-size: 1.75rem;
            margin-bottom: 1.5rem;
            color: #2d3748;
            border-bottom: 3px solid #667eea;
            padding-bottom: 0.5rem;
        }}
        .advice-box {{
            background: linear-gradient(135deg, #f6f8fb 0%, #e9ecef 100%);
            border-left: 4px solid #667eea;
            padding: 1.5rem;
            border-radius: 8px;
            font-family: 'Courier New', monospace;
            white-space: pre-wrap;
            line-height: 1.6;
            color: #2d3748;
        }}
        .chart-container {{
            position: relative;
            height: 400px;
            margin-top: 1.5rem;
        }}
        .metrics-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 1.5rem;
            margin-top: 1.5rem;
        }}
        .metric-card {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 1.5rem;
            border-radius: 12px;
            box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
        }}
        .metric-label {{
            font-size: 0.875rem;
            opacity: 0.9;
            margin-bottom: 0.5rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }}
        .metric-value {{
            font-size: 2rem;
            font-weight: bold;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 1.5rem;
        }}
        th, td {{
            padding: 1rem;
            text-align: left;
            border-bottom: 1px solid #e2e8f0;
        }}
        th {{
            background: #f7fafc;
            font-weight: 600;
            color: #2d3748;
        }}
        tr:hover {{
            background: #f7fafc;
        }}
        .badge {{
            display: inline-block;
            padding: 0.25rem 0.75rem;
            border-radius: 12px;
            font-size: 0.875rem;
            font-weight: 600;
        }}
        .badge-success {{ background: #c6f6d5; color: #22543d; }}
        .badge-warning {{ background: #feebc8; color: #7c2d12; }}
        .badge-danger {{ background: #fed7d7; color: #742a2a; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Financial Life Simulation Report</h1>
            <p class="subtitle">Comprehensive lifepath analysis with real-world constraints</p>
        </div>

        <div class="card">
            <h2>Recommended Strategy</h2>
            <div class="advice-box">{}</div>
        </div>

        {}

        {}

        {}
    </div>

    <script>
        const summaryData = {};
        const runsData = {};

        {}
    </script>
</body>
</html>"#,
        advice,
        if has_timeseries {
            r#"<div class="card">
            <h2>Net Worth Distribution Over Time (Funnel Chart)</h2>
            <div class="chart-container">
                <canvas id="funnelChart"></canvas>
            </div>
        </div>"#
        } else {
            ""
        },
        if summary.is_some() {
            r#"<div class="card">
            <h2>Key Metrics at Retirement</h2>
            <div class="metrics-grid" id="metricsGrid"></div>
        </div>"#
        } else {
            ""
        },
        if runs.len() > 1 {
            r#"<div class="card">
            <h2>Strategy Comparison</h2>
            <div class="chart-container">
                <canvas id="strategyScatter"></canvas>
            </div>
            <table id="strategyTable">
                <thead>
                    <tr>
                        <th>Strategy</th>
                        <th>Median Net Worth</th>
                        <th>Success Rate</th>
                        <th>Bankruptcy Rate</th>
                        <th>Score</th>
                    </tr>
                </thead>
                <tbody id="strategyTableBody"></tbody>
            </table>
        </div>"#
        } else {
            ""
        },
        summary_json,
        runs_json,
        js_code
    )
}
