#![cfg(not(target_arch = "wasm32"))]

mod data;
mod rules;
mod transform_rules;
mod worldgen;

use data::game_desc::GameDesc;
use futures_util::lock::Mutex;
use futures_util::{future, SinkExt, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Handle;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use transform_rules::Rules;
use worldgen::galaxy_gen::{create_galaxy, find_stars};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    println!("Starting...");
    let listener = TcpListener::bind("127.0.0.1:62879").await?;
    println!("Started.");
    println!("You may now turn on native mode to search.");
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum IncomingMessage {
    Generate {
        game: GameDesc,
    },
    Find {
        game: GameDesc,
        rule: Rules,
        range: (i32, i32),
        concurrency: i32,
        autosave: u64,
    },
    Stop,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum OutgoingMessage {
    Result { seed: i32, indexes: Vec<usize> },
    Progress { start: i32, end: i32 },
    Done { start: i32, end: i32 },
}

struct FindState {
    pub progress_start: i32,
    pub progress_end: i32,
    pub pending_seeds: HashSet<i32>,
    pub running: i32,
    pub autosave: u64,
    pub last_notify: SystemTime,
}

impl FindState {
    pub fn add(&mut self, seed: i32) -> Option<(i32, i32)> {
        if self.progress_end == seed {
            self.progress_end += 1;
            let mut e: i32 = self.progress_end;
            while self.pending_seeds.remove(&e) {
                e += 1;
            }
            self.progress_end = e;
            let now: SystemTime = SystemTime::now();
            if now.duration_since(self.last_notify).unwrap().as_secs() >= self.autosave {
                self.last_notify = now;
                let start = self.progress_start;
                self.progress_start = self.progress_end;
                Some((start, self.progress_end))
            } else {
                None
            }
        } else {
            self.pending_seeds.insert(seed);
            None
        }
    }
}

async fn accept_connection(stream: TcpStream) {
    let ws_stream = accept_async(stream)
        .await
        .expect("Error during websocket handshake");
    let (write, read) = ws_stream.split();

    let boxed_write = Arc::new(Mutex::new(write));
    let stopped = Arc::new(AtomicBool::new(false));

    let _ = read
        .try_for_each(|msg| {
            if !msg.is_empty() {
                let msg: IncomingMessage = serde_json::from_str(&msg.to_string()).unwrap();
                match msg {
                    IncomingMessage::Stop => {
                        println!("Stopping");
                        stopped.store(true, Ordering::SeqCst);
                    }
                    IncomingMessage::Generate { game } => {
                        let w = boxed_write.clone();
                        tokio::task::spawn_blocking(move || {
                            let galaxy = create_galaxy(&game);
                            let output = serde_json::to_string(&galaxy).unwrap();
                            let runtime = Handle::current();
                            runtime.block_on(async move {
                                w.lock().await.send(Message::Text(output)).await.unwrap();
                            })
                        });
                    }
                    IncomingMessage::Find {
                        game,
                        rule,
                        range: (start, end),
                        concurrency,
                        autosave,
                    } => {
                        println!("Receive search request.");
                        println!("Concurrency: {}.", concurrency);
                        let threads = concurrency.min(end - start);
                        let current_seed = Arc::new(AtomicI32::new(start));
                        let state = Arc::new(std::sync::Mutex::new(FindState {
                            progress_end: start,
                            progress_start: start,
                            running: threads,
                            pending_seeds: HashSet::new(),
                            autosave,
                            last_notify: SystemTime::now(),
                        }));
                        stopped.store(false, Ordering::SeqCst);
                        for _ in 0..threads {
                            let w = boxed_write.clone();
                            let mut transformed = transform_rules::transform_rules(rule.clone());
                            let mut g = game.clone();
                            let s = state.clone();
                            let cs = current_seed.clone();
                            let stop = stopped.clone();
                            tokio::task::spawn_blocking(move || {
                                let runtime = Handle::current();
                                loop {
                                    let seed = cs
                                        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
                                            Some(x + 1)
                                        })
                                        .unwrap();
                                    if seed >= end {
                                        break;
                                    }
                                    g.seed = seed;
                                    let star_indexes = find_stars(&g, &mut transformed);
                                    let notify_progress = {
                                        let mut x = s.lock().unwrap();
                                        x.add(seed)
                                    };
                                    if notify_progress.is_some() || !star_indexes.is_empty() {
                                        let w2 = w.clone();
                                        runtime.block_on(async move {
                                            let mut stream = w2.lock().await;
                                            if !star_indexes.is_empty() {
                                                let output = serde_json::to_string(
                                                    &OutgoingMessage::Result {
                                                        seed,
                                                        indexes: star_indexes,
                                                    },
                                                )
                                                .unwrap();
                                                stream.send(Message::Text(output)).await.unwrap();
                                            }
                                            if let Some((start, end)) = notify_progress {
                                                println!("Processing: {}.", end);
                                                let output = serde_json::to_string(
                                                    &OutgoingMessage::Progress { start, end },
                                                )
                                                .unwrap();
                                                stream.send(Message::Text(output)).await.unwrap();
                                            }
                                        });
                                    }
                                    if stop.load(Ordering::SeqCst) {
                                        break;
                                    }
                                }
                                let mut x = s.lock().unwrap();
                                x.running -= 1;
                                if x.running == 0 {
                                    let progress_start = x.progress_start;
                                    let progress_end = x.progress_end;
                                    println!("Completed: {}.", progress_end);
                                    runtime.block_on(async move {
                                        w.lock()
                                            .await
                                            .send(Message::Text(
                                                serde_json::to_string(&OutgoingMessage::Done {
                                                    start: progress_start,
                                                    end: progress_end,
                                                })
                                                .unwrap(),
                                            ))
                                            .await
                                            .unwrap();
                                    })
                                }
                            });
                        }
                    }
                }
            }
            future::ok(())
        })
        .await;
}
