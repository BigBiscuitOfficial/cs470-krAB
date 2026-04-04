#[cfg(feature = "distributed_mpi")]
use krabmaga::engine::run::RunStats;
#[cfg(feature = "distributed_mpi")]
use krabmaga::explore::local_sweep::SweepRecord;
#[cfg(feature = "distributed_mpi")]
use krabmaga::explore::mpi::sweep::sort_records_by_job_id;

#[cfg(feature = "distributed_mpi")]
#[test]
fn mpi_sweep_sort_orders_by_conf_and_rep_id() {
    let stats = RunStats {
        run_duration: 0.0,
        executed_steps: 0,
    };

    let mut records = vec![
        SweepRecord {
            conf_id: 2,
            rep_id: 1,
            config: 2usize,
            result: 21i32,
            stats,
        },
        SweepRecord {
            conf_id: 1,
            rep_id: 3,
            config: 1usize,
            result: 13i32,
            stats,
        },
        SweepRecord {
            conf_id: 1,
            rep_id: 0,
            config: 1usize,
            result: 10i32,
            stats,
        },
        SweepRecord {
            conf_id: 2,
            rep_id: 0,
            config: 2usize,
            result: 20i32,
            stats,
        },
    ];

    sort_records_by_job_id(&mut records);

    let ids: Vec<(u64, u32)> = records.iter().map(|r| (r.conf_id, r.rep_id)).collect();
    assert_eq!(ids, vec![(1, 0), (1, 3), (2, 0), (2, 1)]);
}
