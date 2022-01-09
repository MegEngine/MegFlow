/**
 * \file flow-rs/src/broker.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use crate::envelope::*;
use crate::rt::channel::*;
use crate::rt::task::JoinHandle;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

type NotifyReceiver = Receiver<SealedEnvelope>;
type NotifySender = Sender<SealedEnvelope>;
type SubReceiver = Receiver<SealedEnvelope>;
type SubSender = Sender<SealedEnvelope>;

#[derive(Default)]
pub struct Broker {
    subs: HashMap<String, (NotifySender, NotifyReceiver, Vec<SubSender>)>,
    running: Arc<AtomicBool>,
    count: Arc<AtomicUsize>,
}

pub struct BrokerClient {
    id: usize,
    notify: NotifySender,
    self_notify: SubSender,
    sub: SubReceiver,
    topic: String,
    running: Arc<AtomicBool>,
    count: Arc<AtomicUsize>,
}

impl Broker {
    pub fn new() -> Broker {
        Broker {
            subs: HashMap::new(),
            running: Arc::new(AtomicBool::new(false)),
            count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn subscribe(&mut self, topic: String) -> BrokerClient {
        let (notify, _, subs) = self.subs.entry(topic.clone()).or_insert_with(|| {
            let (s, r) = unbounded();
            (s, r, vec![])
        });
        let (s, r) = unbounded();
        subs.push(s.clone());
        self.count.fetch_add(1, Ordering::Relaxed);
        BrokerClient {
            id: subs.len() - 1,
            notify: notify.clone(),
            self_notify: s,
            sub: r,
            topic,
            running: self.running.clone(),
            count: self.count.clone(),
        }
    }

    pub fn run(&mut self) -> JoinHandle<anyhow::Result<()>> {
        self.running.store(true, Ordering::Relaxed);
        let running = self.running.clone();
        let subs = std::mem::take(&mut self.subs);
        crate::rt::task::spawn(async move {
            let mut handles = vec![];
            for (_, (_, notifier, subs)) in subs.into_iter() {
                let running = running.clone();
                handles.push(crate::rt::task::spawn(async move {
                    while running.load(Ordering::Acquire) {
                        if let Ok(msg) = notifier.recv().await {
                            for (i, sub) in subs.iter().enumerate() {
                                if i == msg
                                    .info()
                                    .from_addr
                                    .expect("message from address is lost in broker")
                                    as usize
                                {
                                    continue;
                                }
                                let msg = msg.clone();
                                sub.send(msg).await.unwrap();
                            }
                        }
                    }
                }));
            }
            for handle in handles {
                handle.await;
            }
            Ok(())
        })
    }
}

impl Drop for BrokerClient {
    fn drop(&mut self) {
        if self.count.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.close();
        }
    }
}

impl BrokerClient {
    pub async fn publish<T>(&self, msg: T)
    where
        T: 'static + Clone + std::marker::Send,
    {
        let mut envelope = Envelope::new(msg.clone()).seal();
        envelope.info_mut().from_addr = Some(self.id as u64);
        let envelope_cloned = envelope.clone();
        self.notify.send(envelope_cloned).await.unwrap();
        self.self_notify.send(envelope).await.unwrap();
    }

    pub async fn fetch<T>(&self) -> Result<T, RecvError>
    where
        T: 'static + Clone + std::marker::Send,
    {
        self.sub.recv().await.map(|mut envelope| {
            let envelope = envelope
                .downcast_mut::<Envelope<T>>()
                .expect("type error when downcast");
            envelope.unpack()
        })
    }

    pub fn try_fetch<T>(&self) -> Option<T>
    where
        T: 'static + Clone + std::marker::Send,
    {
        self.sub.try_recv().ok().map(|mut envelope| {
            let envelope = envelope
                .downcast_mut::<Envelope<T>>()
                .expect("type error when downcast");
            envelope.unpack()
        })
    }

    pub fn topic(&self) -> &str {
        self.topic.as_str()
    }

    pub fn is_closed(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn close(&self) {
        self.running.store(false, Ordering::Release);
        self.notify.close();
        self.sub.close();
    }
}
