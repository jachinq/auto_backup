use std::{
    str::FromStr,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
};

// use cron_job::{CronJob, Job};

use crate::{
    backup::{self, Backup},
    data::{AutoStatus, Data, SaveItem, Status},
    log,
};

#[derive(Debug)]
pub enum DataSignal {
    Terminated,
    Sync(String, Backup),
}

#[derive(Debug, Default, Clone)]
pub struct StartInfo {
    pub success: bool,
    pub save_item: SaveItem,
    pub error: Option<String>,
}

pub fn receive_data_signal(
    data: Arc<Mutex<Data>>,
    active: Arc<Mutex<SaveItem>>,
    job_rx: Receiver<DataSignal>,
) {
    let _ = thread::spawn(move || loop {
        if let Ok(signal) = job_rx.recv() {
            // println!("got a signal = {:?}", signal);
            match signal {
                DataSignal::Terminated => return,
                DataSignal::Sync(id, backup) => {
                    if let Ok(mut data) = data.lock() {
                        for item in &mut data.monitors {
                            if id == item.id {
                                item.backups.insert(0, backup.clone());
                                data.save();
                                break;
                            }
                        }
                    }
                    if let Ok(mut active) = active.lock() {
                        active.backups.insert(0, backup);
                    }
                }
            }
        }
    });
}

#[derive(Clone)]
pub struct BackupJob {
    save_item: SaveItem,
    sender: Sender<DataSignal>,
}

impl Job for BackupJob {
    fn run(&mut self) {
        log::log_time(format!(
            "job start, name={} monitros={:?}",
            self.save_item.name, self.save_item.monitors
        ));
        let id = self.save_item.id.to_string();
        let remark = "Auto Backup".to_string();
        let backup =
            backup::backup_file(&id, remark, self.save_item.monitors.to_vec(), &mut vec![]);
        if let Err(e) = self.sender.send(DataSignal::Sync(id, backup)) {
            log::log_err(format!(
                "auto backup success, but associate to data error; e: {}",
                e
            ));
            // println!("auto bk err {}", e);
        }
    }
}

pub struct JobHandle {
    cron_job: Arc<Mutex<CronJob>>,
    handle: Option<Sender<Signal>>,
    sender: Option<Sender<DataSignal>>,
    pub start_infos: Arc<Mutex<Vec<StartInfo>>>,
}

impl Default for JobHandle {
    fn default() -> Self {
        Self {
            cron_job: Default::default(),
            handle: None,
            sender: None,
            start_infos: Default::default(),
        }
    }
}

impl JobHandle {
    pub fn stop_job(&mut self) {
        if let Some(tx) = &self.handle {
            let _ = tx.send(Signal::Stop);
        }
        if let Some(tx) = &self.sender {
            let _ = tx.send(DataSignal::Terminated); // 发送终止信号，停掉上一个监听线程
                                                     // println!("teminate;");
        }
    }
    pub fn start_job(&mut self, jobs: Vec<SaveItem>) -> Receiver<DataSignal> {
        self.stop_job();
        let (tx, rx) = std::sync::mpsc::channel();

        let arc_cron_job = self.cron_job.clone();

        let (job_tx, job_rx) = std::sync::mpsc::channel();

        let job_tx_thread = job_tx.clone();

        let vec = Vec::new();
        let start_infos = Arc::new(Mutex::new(vec));
        let start_infos_clone = start_infos.clone();

        let _ = thread::spawn(move || {
            if let Ok(mut mutex_cron_job) = arc_cron_job.lock() {
                for job in &jobs {
                    if job.status == Status::Archive
                        || !job.auto.open
                        || job.auto.status != AutoStatus::Running
                    {
                        // println!("auto = {:?}", job.auto);
                        continue;
                    }
                    let expression = job.auto.cron.to_string();
                    match cron::Schedule::from_str(&expression) {
                        Ok(_shedule) => {
                            let backup_job = BackupJob {
                                save_item: job.clone(),
                                sender: job_tx_thread.clone(),
                            };
                            mutex_cron_job.new_job(&expression, backup_job);
                            start_infos.lock().unwrap().push(StartInfo {
                                success: true,
                                save_item: job.clone(),
                                error: None,
                            })
                        }
                        Err(e) => {
                            let error = format!("has err = {}", e);
                            start_infos.lock().unwrap().push(StartInfo {
                                success: false,
                                save_item: job.clone(),
                                error: Some(error),
                            })
                        }
                    }
                }
                mutex_cron_job.start(rx);
            } else {
                log::log_err("start cron job error, get lock error.");
            }
        });

        // self.info.push(JobInfo { name: job.name.to_string(), error: None });
        self.handle = Some(tx);
        self.sender = Some(job_tx);
        self.start_infos = start_infos_clone;
        job_rx
    }
}

/// basic
use chrono::{DateTime, FixedOffset, Local};
use cron::Schedule;

use std::thread;
use std::time::Duration;

/// The struct to create and execute all the cronjobs.
struct CronJob {
    jobs: Vec<BackupJob>,
    expressions: Vec<String>,
    offset: Option<FixedOffset>,
    interval: u64,
}

#[derive(Debug, PartialEq)]
enum Signal {
    Stop,
}

#[allow(unused)]
impl CronJob {
    /// Constructs new `CronJob` object.
    pub fn new(offset: Option<FixedOffset>, interval: u64) -> Self {
        CronJob {
            jobs: Vec::new(),
            expressions: Vec::new(),
            offset,
            interval,
        }
    }

    /// Sets the interval for the cronjobs.
    pub fn set_interval(&mut self, interval: u64) {
        self.interval = interval;
    }

    /// Sets the offset for the cronjobs.
    pub fn set_offset(&mut self, offset: FixedOffset) {
        self.offset = Some(offset);
    }

    /// Returns the schedules for all the cronjobs, with this you are able to get the next occurrences.
    pub fn get_schedules(&self) -> Vec<Schedule> {
        self.expressions
            .iter()
            .map(|ex| Schedule::from_str(ex).unwrap())
            .collect()
    }

    /// Allows to add a new job to the cronjobs.
    pub fn new_job(&mut self, expression: &str, job: BackupJob) {
        self.expressions.push(expression.to_string());
        self.jobs.push(job);
    }

    /// Stop job,this will be clear all job
    pub fn stop(&mut self) {
        self.expressions.clear();
        self.jobs.clear();
    }

    /// Starts the cronjobs without threading.
    pub fn start(&mut self, rx: Receiver<Signal>) {
        let schedules = self.get_schedules();
        let offset = self
            .offset
            .unwrap_or_else(|| FixedOffset::east_opt(0).unwrap());

        loop {
            if let Ok(status) = rx.try_recv() {
                if status == Signal::Stop {
                    self.stop();
                    return;
                }
            }

            let upcomings: Vec<Option<DateTime<FixedOffset>>> = schedules
                .iter()
                .map(|schedule| schedule.upcoming(offset).take(1).next())
                .collect();
            thread::sleep(Duration::from_millis(self.interval));
            let local = &Local::now();

            for (i, upcoming) in upcomings.iter().enumerate() {
                if let Some(datetime) = upcoming {
                    if datetime.timestamp() <= local.timestamp() {
                        let mut job = self.jobs[i].clone();
                        std::thread::spawn(move || job.run());
                        // self.jobs[i].run()
                    }
                }
            }
        }
    }
}

/// Default implementation for CronJob.
impl Default for CronJob {
    fn default() -> Self {
        Self {
            jobs: vec![],
            expressions: Vec::new(),
            offset: None,
            interval: 500,
        }
    }
}

/// The Job trait, allows structs to be run as cronjobs.
trait Job: Sync + Send + Clone + 'static {
    fn run(&mut self);
}

pub fn parse_time(cron: &str) -> Vec<String> {
    if let Ok(sc) = cron::Schedule::from_str(cron) {
        sc.upcoming(Local)
            .take(5)
            .map(|v| v.format("%Y-%m-%d %H:%M:%S").to_string())
            .collect()
    } else {
        vec![]
    }
}
