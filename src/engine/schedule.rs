extern crate priority_queue;

use crate::engine::{agent::Agent, agentimpl::AgentImpl, priority::Priority, state::State};

use cfg_if::cfg_if;
use priority_queue::PriorityQueue;
use std::fmt;

cfg_if! {
    if #[cfg(feature ="parallel")]{
        use crossbeam::thread;
        use std::sync::{Arc,Mutex};
        use clap::{App, Arg};
        use lazy_static::*;

    }
}

cfg_if! {
    if #[cfg(feature ="parallel")]{
        lazy_static! {
            pub static ref THREAD_NUM: usize = {
                let matches = App::new("krABMaga")
                    .arg(Arg::with_name("bench").long("bench"))
                    .arg(
                        Arg::with_name("num_thread")
                            .help("sets the number of threads to use")
                            .takes_value(true)
                            .long("nt"),
                    )
                    .get_matches();
                let n = match matches.value_of("num_thread") {
                    Some(nt) => match nt.parse::<usize>() {
                        Ok(ris) => ris,
                        Err(_) => {
                            eprintln!("error: --nt value is not an integer");
                            num_cpus::get()
                        }
                    },
                    _ => 1,
                };
                n
            };
        }
    }
}
cfg_if! {
    if #[cfg(feature ="parallel")] {
        pub struct Schedule {
            pub step: usize,
            pub time: f32,
            pub events: Arc<Mutex<PriorityQueue<AgentImpl, Priority>>>,
            pub thread_num:usize,
            pub agent_ids_counting: Arc<Mutex<u32>>,
        }

        #[derive(Clone)]
        pub struct Pair {
            agentimpl: AgentImpl,
            priority: Priority,
        }

        impl Pair {
            #[allow(dead_code)]
            fn new(agent: AgentImpl, the_priority: Priority) -> Pair {
                Pair {
                    agentimpl: agent,
                    priority: the_priority
                }
            }
        }

        impl fmt::Display for Pair {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "agent: {} priority: {}", self.agentimpl, self.priority)
            }
        }

        impl Schedule {
            // MT_OPT_BEGIN: adaptive-threshold
            const MT_FAST_PATH_MIN_EVENTS: usize = 64;
            // MT_OPT_END: adaptive-threshold

            pub fn new() -> Schedule {
                Schedule {
                    step: 0,
                    time: 0.0,
                    events: Arc::new(Mutex::new(PriorityQueue::new())),
                    thread_num: *THREAD_NUM,
                    agent_ids_counting: Arc::new(Mutex::new(0)),
                }
            }

            pub fn with_threads(thread_num: usize) -> Schedule {
                Schedule {
                    step: 0,
                    time: 0.0,
                    events: Arc::new(Mutex::new(PriorityQueue::new())),
                    thread_num,
                    agent_ids_counting: Arc::new(Mutex::new(0)),
                }
            }

            pub fn schedule_once(&mut self, agent: AgentImpl, the_time:f32, the_ordering:i32) {
                self.events.lock().expect("error on lock").push(
                    agent,
                    Priority {
                        time: the_time,
                        ordering: the_ordering,
                    },
                );
            }

            pub fn schedule_repeating(&mut self, agent: Box<dyn Agent>, the_time:f32, the_ordering:i32) {
                let mut agent_ids_counting = self.agent_ids_counting.lock().expect("error on lock");
                let mut a = AgentImpl::new(agent, *agent_ids_counting);
                *agent_ids_counting +=1;
                a.repeating = true;
                let pr = Priority::new(the_time, the_ordering);
                self.events.lock().expect("error on lock").push(a, pr);
            }

            pub fn get_all_events(&self) -> Vec<Box<dyn Agent>>{
                let mut tor: Vec<Box<dyn Agent>> = Vec::new();
                for e in self.events.lock().expect("error on lock").iter(){
                    tor.push(e.0.agent.clone());
                }
                tor
            }

            pub fn step(&mut self, state: &mut dyn State) {
                if self.step == 0{
                    state.update(self.step as u64);
                }

                state.before_step(self);

                // MT_OPT_BEGIN: queue-drain-single-lock
                let mut drained_events: Vec<Pair> = Vec::new();
                {
                    let mut queue = self.events.lock().expect("error on lock");
                    if queue.is_empty() {
                        println!("No agent in the queue to schedule. Terminating.");
                        //TODO check if we need to exit on 0 agents or we have to continue until new agents are spawned
                        std::process::exit(0);
                    }

                    match queue.peek() {
                        Some(item) => {
                            let (_agent, priority) = item;
                            self.time = priority.time;
                        },
                        None => panic!("Agent not found - out loop")
                    }

                    drained_events.reserve(queue.len());
                    while let Some((agent, priority)) = queue.pop() {
                        drained_events.push(Pair::new(agent, priority));
                    }
                }
                // MT_OPT_END: queue-drain-single-lock

                let thread_num = self.thread_num.max(1);

                // MT_OPT_BEGIN: adaptive-fast-path
                let use_parallel =
                    thread_num > 1 && drained_events.len() >= Self::MT_FAST_PATH_MIN_EVENTS;

                if !use_parallel {
                    let mut rescheduled = Vec::new();

                    for mut item in drained_events.into_iter() {
                        item.agentimpl.agent.before_step(state);
                        item.agentimpl.agent.step(state);
                        item.agentimpl.agent.after_step(state);

                        if item.agentimpl.repeating && !item.agentimpl.agent.is_stopped(state) {
                            rescheduled.push((
                                item.agentimpl,
                                Priority {
                                    time: item.priority.time + 1.0,
                                    ordering: item.priority.ordering,
                                },
                            ));
                        }
                    }

                    if !rescheduled.is_empty() {
                        let mut queue = self.events.lock().expect("error on lock");
                        for (agent, priority) in rescheduled.into_iter() {
                            queue.push(agent, priority);
                        }
                    }
                } else {
                    // MT_OPT_END: adaptive-fast-path
                    // MT_OPT_BEGIN: batched-reschedule-locks
                    let mut cevents: Vec<Vec<Pair>> = (0..thread_num).map(|_| Vec::new()).collect();
                    for (i, item) in drained_events.into_iter().enumerate() {
                        cevents[i % thread_num].push(item);
                    }

                    let state = Arc::new(Mutex::new(state));
                    let _result = thread::scope( |scope| {
                        for mut batch in cevents.into_iter() {
                            if batch.is_empty() {
                                continue;
                            }

                            let events = Arc::clone(&self.events);
                            let state = Arc::clone(&state);

                            scope.spawn(move |_| {
                                let mut rescheduled = Vec::new();

                                for mut item in batch.drain(..){
                                    let mut state = state.lock().expect("error on lock");
                                    let state = state.as_state_mut();

                                    item.agentimpl.agent.before_step(state);
                                    item.agentimpl.agent.step(state);
                                    item.agentimpl.agent.after_step(state);

                                    if item.agentimpl.repeating && !item.agentimpl.agent.is_stopped(state) {
                                        rescheduled.push((
                                            item.agentimpl,
                                            Priority {
                                                time: item.priority.time + 1.0,
                                                ordering: item.priority.ordering,
                                            },
                                        ));
                                    }
                                }

                                if !rescheduled.is_empty() {
                                    let mut q = events.lock().expect("error on lock");
                                    for (agent, priority) in rescheduled.into_iter() {
                                        q.push(agent, priority);
                                    }
                                }
                            });
                        }
                    });

                    let mut state_guard = state.lock().expect("error on lock");
                    state_guard.as_state_mut().after_step(self);
                    self.step += 1;
                    state_guard.as_state_mut().update(self.step as u64);
                    return;
                    // MT_OPT_END: batched-reschedule-locks
                }

                state.after_step(self);
                self.step += 1;
                state.update(self.step as u64);
            }
        }
    }
    // SEQUENTIAL IF
    else{
        /// Struct to manage all the agents in the simulation
        pub struct Schedule{
            /// Current step of the simulation
            pub step: u64,
            /// Current time of the simulation
            pub time: f32,
            /// Priority queue filled with a pair of AgentImpl and his Priority
            pub events: PriorityQueue<AgentImpl,Priority>,
            /// Unique ids inside schedule
            pub agent_ids_counting: u32,
        }

        /// internal struct to manage the AgentImpl in a more convenient way
        #[derive(Clone)]
        struct Pair{
            agentimpl: AgentImpl,
            priority: Priority,
        }

        impl Pair {
            fn new(agent: AgentImpl, the_priority: Priority) -> Pair {
                Pair {
                    agentimpl: agent,
                    priority: the_priority
                }
            }
        }

        impl fmt::Display for Pair {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "agent: {} priority: {}", self.agentimpl, self.priority)
            }
        }

        impl Default for Schedule {
            fn default() -> Self {
                Self::new()
            }
        }

        impl Schedule {
            /// Create a new instance for Schedule
            pub fn new() -> Schedule {
                Schedule {
                    step: 0,
                    time: 0.0,
                    events: PriorityQueue::new(),
                    agent_ids_counting: 0,
                }
            }

            /// Onsert an agent in the PriorityQueue for one step
            ///
            /// # Arguments
            /// * `agent` - Agent to schedule
            /// * `the_time` - Time to schedule the agent
            /// * `the_ordering` - Ordering of the agent inside the queue
            pub fn schedule_once(&mut self, agent: AgentImpl, the_time:f32, the_ordering:i32) {
                self.events.push(agent, Priority{time: the_time, ordering: the_ordering});
            }

            /// Insert an agent in the PriorityQueue with the repeating field set at true.
            ///
            /// Return false if the insertion in the priority queue fails.
            ///
            /// # Arguments
            /// * `agent` - Agent to schedule
            /// * `the_time` - Time to schedule the agent
            /// * `the_ordering` - Ordering of the agent inside the queue
            pub fn schedule_repeating(&mut self, agent: Box<dyn Agent>, the_time:f32, the_ordering:i32) -> bool {
                let mut a = AgentImpl::new(agent, self.agent_ids_counting);
                self.agent_ids_counting +=1;
                a.repeating = true;

                let pr = Priority::new(the_time, the_ordering);
                let opt = self.events.push(a, pr);
                opt.is_none()
            }

            pub fn distributed_schedule_repeating(&mut self, agent: Box<dyn Agent>, the_time:f32, the_ordering:i32) -> (u32,bool) {
                let mut a = AgentImpl::new(agent, self.agent_ids_counting);
                self.agent_ids_counting +=1;
                a.repeating = true;

                let pr = Priority::new(the_time, the_ordering);
                let opt = self.events.push(a, pr);
                (self.agent_ids_counting-1, opt.is_none())
            }

            /// Return a vector of all the objects contained in the PriorityQueue
            pub fn get_all_events(&self) -> Vec<Box<dyn Agent>>{
                let mut tor: Vec<Box<dyn Agent>> = Vec::new();
                for e in self.events.iter(){
                    tor.push(e.0.agent.clone());
                }
                tor
            }

            /// Remove an agent, if exist, from the PriorityQueue.
            ///
            /// # Arguments
            /// * `agent` - Agent to remove
            /// * `my_id` - Id of the agent to remove
            pub fn dequeue(&mut self, agent: Box<dyn Agent>, my_id: u32) -> bool {
                let a = AgentImpl::new(agent, my_id);
                let removed = self.events.remove(&a);
                match removed {
                    //some if found and removed
                    Some(_) => {

                        // println!("Agent {} -- {} removed from the queue",a, my_id);
                        true
                    },
                    None => false,
                }
            }

            /// Compute the step for each agent in the PriorityQueue.
            ///
            /// # Arguments
            /// * `state` - State of the simulation
            pub fn step(&mut self, state: &mut dyn State){

                if self.step == 0{
                    state.update(self.step);
                }

                state.before_step(self);

                let events = &mut self.events;

                if events.is_empty() {
                    println!("No agent in the queue to schedule. Terminating.");
                    //TODO check if we need to exit on 0 agents or we have to continue until new agents are spawned
                    // std::process::exit(0);
                    state.after_step(self);
                    self.step += 1;
                    state.update(self.step);
                    return;
                }

                let mut cevents: Vec<Pair> = Vec::new();

                match events.peek() {
                    Some(item) => {
                        let (_agent, priority) = item;
                        self.time = priority.time;
                    },
                    None => panic!("Agent not found - out loop"),
                }

                loop {
                    if events.is_empty() {
                        break;
                    }

                    match events.peek() {
                        Some(item) => {
                            let (_, priority) = item;
                            if priority.time > self.time {
                                break;
                            }
                            let (agent, priority) = events.pop().expect("Error on pop from queue");
                            cevents.push(Pair::new(agent, priority));
                        },
                        None => panic!("Agent not found - inside loop"),
                    }
                }

                for mut item in cevents.into_iter() {

                    item.agentimpl.agent.before_step(state);
                    item.agentimpl.agent.step(state);
                    item.agentimpl.agent.after_step(state);

                    if item.agentimpl.repeating && !item.agentimpl.agent.is_stopped(state) {
                        self.schedule_once(
                            item.agentimpl,
                            item.priority.time + 1.0,
                            item.priority.ordering,
                        );
                    }
                }

                state.after_step(self);
                self.step += 1;
                state.update(self.step);
            }
        }
    }
}

#[doc(hidden)]
/// A struct used to specify schedule options to pass to an agent's clone when an agent reproduces.
pub struct ScheduleOptions {
    pub ordering: i32,
    pub repeating: bool,
}
