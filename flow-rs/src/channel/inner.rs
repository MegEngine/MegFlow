/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * License); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * AS IS BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

/*
 * Copyright (c)  https://github.com/smol-rs/async-channel
 */

/*
 * Copyright (c) 2021, megvii. inc
 * Author: MegEngine@megvii.com
 */

use std::error;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::usize;

use concurrent_queue::{ConcurrentQueue, PopError, PushError};
use event_listener::{Event, EventListener};
use futures_core::stream::Stream;

pub struct Channel<T> {
    /// Inner message queue.
    pub(super) queue: ConcurrentQueue<T>,

    /// Send operations waiting while the channel is full.
    send_ops: Event,

    /// Receive operations waiting while the channel is empty and not closed.
    recv_ops: Event,

    /// Stream operations while the channel is empty and not closed.
    stream_ops: Event,

    /// The number of currently active `Sender`s.
    pub(super) sender_count: AtomicUsize,

    /// The number of currently active `Receivers`s.
    pub(super) receiver_count: AtomicUsize,
}

impl<T> Channel<T> {
    /// Closes the channel and notifies all blocked operations.
    ///
    /// Returns `true` if this call has closed the channel and it was not closed already.
    pub(super) fn close(&self) -> bool {
        if self.queue.close() {
            // Notify all send operations.
            self.send_ops.notify(usize::MAX);

            // Notify all receive and stream operations.
            self.recv_ops.notify(usize::MAX);
            self.stream_ops.notify(usize::MAX);

            true
        } else {
            false
        }
    }
}

pub fn bounded<T>(cap: usize) -> Arc<Channel<T>> {
    assert!(cap > 0, "capacity cannot be zero");

    Arc::new(Channel {
        queue: ConcurrentQueue::bounded(cap),
        send_ops: Event::new(),
        recv_ops: Event::new(),
        stream_ops: Event::new(),
        sender_count: AtomicUsize::new(0),
        receiver_count: AtomicUsize::new(0),
    })
}

pub fn unbounded<T>() -> Arc<Channel<T>> {
    Arc::new(Channel {
        queue: ConcurrentQueue::unbounded(),
        send_ops: Event::new(),
        recv_ops: Event::new(),
        stream_ops: Event::new(),
        sender_count: AtomicUsize::new(0),
        receiver_count: AtomicUsize::new(0),
    })
}

/// The sending side of a channel.
///
/// Senders can be cloned and shared among threads. When all senders associated with a channel are
/// dropped, the channel becomes closed.
///
/// The channel can also be closed manually by calling [`Sender::close()`].
pub struct Sender<T> {
    /// Inner channel state.
    pub(super) channel: Arc<Channel<T>>,
    pub(super) close_ops: Arc<Event>,
}

impl<T> Sender<T> {
    pub fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        match self.channel.queue.push(msg) {
            Ok(()) => {
                // Notify a single blocked receive operation. If the notified operation then
                // receives a message or gets canceled, it will notify another blocked receive
                // operation.
                self.channel.recv_ops.notify(1);

                // Notify all blocked streams.
                self.channel.stream_ops.notify(usize::MAX);

                Ok(())
            }
            Err(PushError::Full(msg)) => Err(TrySendError::Full(msg)),
            Err(PushError::Closed(msg)) => Err(TrySendError::Closed(msg)),
        }
    }

    pub async fn send(&self, msg: T) -> Result<(), SendError<T>> {
        let mut listener = None;
        let mut msg = msg;

        loop {
            // Attempt to send a message.
            match self.try_send(msg) {
                Ok(()) => {
                    // If the capacity is larger than 1, notify another blocked send operation.
                    match self.channel.queue.capacity() {
                        Some(1) => {}
                        Some(_) | None => self.channel.send_ops.notify(1),
                    }
                    return Ok(());
                }
                Err(TrySendError::Closed(msg)) => return Err(SendError(msg)),
                Err(TrySendError::Full(m)) => msg = m,
            }

            // Sending failed - now start listening for notifications or wait for one.
            match listener.take() {
                None => {
                    // Start listening and then try receiving again.
                    listener = Some(self.channel.send_ops.listen());
                }
                Some(l) => {
                    // Wait for a notification.
                    l.await;
                }
            }
        }
    }

    pub fn close(&self) -> bool {
        self.channel.close()
    }

    pub fn is_closed(&self) -> bool {
        self.channel.queue.is_closed()
    }

    pub fn is_empty(&self) -> bool {
        self.channel.queue.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.channel.queue.is_full()
    }

    pub fn len(&self) -> usize {
        self.channel.queue.len()
    }

    pub fn capacity(&self) -> Option<usize> {
        self.channel.queue.capacity()
    }

    pub fn receiver_count(&self) -> usize {
        self.channel.receiver_count.load(Ordering::SeqCst)
    }

    pub fn sender_count(&self) -> usize {
        self.channel.sender_count.load(Ordering::SeqCst)
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        // Decrement the sender count and close the channel if it drops down to zero.
        if self.channel.sender_count.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.channel.close();
            self.close_ops.notify(usize::MAX);
        }
    }
}

impl<T> fmt::Debug for Sender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sender {{ .. }}")
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Sender<T> {
        let count = self.channel.sender_count.fetch_add(1, Ordering::Relaxed);

        // Make sure the count never overflows, even if lots of sender clones are leaked.
        if count > usize::MAX / 2 {
            process::abort();
        }

        Sender {
            channel: self.channel.clone(),
            close_ops: self.close_ops.clone(),
        }
    }
}

/// The receiving side of a channel.
///
/// Receivers can be cloned and shared among threads. When all receivers associated with a channel
/// are dropped, the channel becomes closed.
///
/// The channel can also be closed manually by calling [`Receiver::close()`].
///
/// Receivers implement the [`Stream`] trait.
pub struct Receiver<T> {
    /// Inner channel state.
    pub(super) channel: Arc<Channel<T>>,

    /// Listens for a send or close event to unblock this stream.
    pub(super) listener: Option<EventListener>,
    pub(super) close_ops: Arc<Event>,
}

impl<T> Receiver<T> {
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        match self.channel.queue.pop() {
            Ok(msg) => {
                // Notify a single blocked send operation. If the notified operation then sends a
                // message or gets canceled, it will notify another blocked send operation.
                self.channel.send_ops.notify(1);

                Ok(msg)
            }
            Err(PopError::Empty) => Err(TryRecvError::Empty),
            Err(PopError::Closed) => Err(TryRecvError::Closed),
        }
    }

    pub async fn recv(&self) -> Result<T, RecvError> {
        let mut listener = None;

        loop {
            // Attempt to receive a message.
            match self.try_recv() {
                Ok(msg) => {
                    // If the capacity is larger than 1, notify another blocked receive operation.
                    // There is no need to notify stream operations because all of them get
                    // notified every time a message is sent into the channel.
                    match self.channel.queue.capacity() {
                        Some(1) => {}
                        Some(_) | None => self.channel.recv_ops.notify(1),
                    }
                    return Ok(msg);
                }
                Err(TryRecvError::Closed) => return Err(RecvError),
                Err(TryRecvError::Empty) => {}
            }

            // Receiving failed - now start listening for notifications or wait for one.
            match listener.take() {
                None => {
                    // Start listening and then try receiving again.
                    listener = Some(self.channel.recv_ops.listen());
                }
                Some(l) => {
                    // Wait for a notification.
                    l.await;
                }
            }
        }
    }

    pub fn close(&self) -> bool {
        self.channel.close()
    }

    pub fn is_closed(&self) -> bool {
        self.channel.queue.is_closed()
    }

    pub fn is_empty(&self) -> bool {
        self.channel.queue.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.channel.queue.is_full()
    }

    pub fn len(&self) -> usize {
        self.channel.queue.len()
    }

    pub fn capacity(&self) -> Option<usize> {
        self.channel.queue.capacity()
    }

    pub fn receiver_count(&self) -> usize {
        self.channel.receiver_count.load(Ordering::SeqCst)
    }

    pub fn sender_count(&self) -> usize {
        self.channel.sender_count.load(Ordering::SeqCst)
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        // Decrement the receiver count and close the channel if it drops down to zero.
        if self.channel.receiver_count.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.channel.close();
            self.close_ops.notify(usize::MAX);
        }
    }
}

impl<T> fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Receiver {{ .. }}")
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Receiver<T> {
        let count = self.channel.receiver_count.fetch_add(1, Ordering::Relaxed);

        // Make sure the count never overflows, even if lots of receiver clones are leaked.
        if count > usize::MAX / 2 {
            process::abort();
        }

        Receiver {
            channel: self.channel.clone(),
            listener: None,
            close_ops: self.close_ops.clone(),
        }
    }
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // If this stream is listening for events, first wait for a notification.
            if let Some(listener) = self.listener.as_mut() {
                futures_core::ready!(Pin::new(listener).poll(cx));
                self.listener = None;
            }

            loop {
                // Attempt to receive a message.
                match self.try_recv() {
                    Ok(msg) => {
                        // The stream is not blocked on an event - drop the listener.
                        self.listener = None;
                        return Poll::Ready(Some(msg));
                    }
                    Err(TryRecvError::Closed) => {
                        // The stream is not blocked on an event - drop the listener.
                        self.listener = None;
                        return Poll::Ready(None);
                    }
                    Err(TryRecvError::Empty) => {}
                }

                // Receiving failed - now start listening for notifications or wait for one.
                match self.listener.as_mut() {
                    None => {
                        // Create a listener and try sending the message again.
                        self.listener = Some(self.channel.stream_ops.listen());
                    }
                    Some(_) => {
                        // Go back to the outer loop to poll the listener.
                        break;
                    }
                }
            }
        }
    }
}

/// An error returned from [`Sender::send()`].
///
/// Received because the channel is closed.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct SendError<T>(pub T);

impl<T> SendError<T> {
    /// Unwraps the message that couldn't be sent.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> error::Error for SendError<T> {}

impl<T> fmt::Debug for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SendError(..)")
    }
}

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sending into a closed channel")
    }
}

/// An error returned from [`Sender::try_send()`].
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TrySendError<T> {
    /// The channel is full but not closed.
    Full(T),

    /// The channel is closed.
    Closed(T),
}

impl<T> TrySendError<T> {
    /// Unwraps the message that couldn't be sent.
    pub fn into_inner(self) -> T {
        match self {
            TrySendError::Full(t) => t,
            TrySendError::Closed(t) => t,
        }
    }

    /// Returns `true` if the channel is full but not closed.
    pub fn is_full(&self) -> bool {
        match self {
            TrySendError::Full(_) => true,
            TrySendError::Closed(_) => false,
        }
    }

    /// Returns `true` if the channel is closed.
    pub fn is_closed(&self) -> bool {
        match self {
            TrySendError::Full(_) => false,
            TrySendError::Closed(_) => true,
        }
    }
}

impl<T> error::Error for TrySendError<T> {}

impl<T> fmt::Debug for TrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TrySendError::Full(..) => write!(f, "Full(..)"),
            TrySendError::Closed(..) => write!(f, "Closed(..)"),
        }
    }
}

impl<T> fmt::Display for TrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TrySendError::Full(..) => write!(f, "sending into a full channel"),
            TrySendError::Closed(..) => write!(f, "sending into a closed channel"),
        }
    }
}

/// An error returned from [`Receiver::recv()`].
///
/// Received because the channel is empty and closed.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct RecvError;

impl error::Error for RecvError {}

impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "receiving from an empty and closed channel")
    }
}

/// An error returned from [`Receiver::try_recv()`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TryRecvError {
    /// The channel is empty but not closed.
    Empty,

    /// The channel is empty and closed.
    Closed,
}

impl TryRecvError {
    /// Returns `true` if the channel is empty but not closed.
    pub fn is_empty(&self) -> bool {
        match self {
            TryRecvError::Empty => true,
            TryRecvError::Closed => false,
        }
    }

    /// Returns `true` if the channel is empty and closed.
    pub fn is_closed(&self) -> bool {
        match self {
            TryRecvError::Empty => false,
            TryRecvError::Closed => true,
        }
    }
}

impl error::Error for TryRecvError {}

impl fmt::Display for TryRecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TryRecvError::Empty => write!(f, "receiving from an empty channel"),
            TryRecvError::Closed => write!(f, "receiving from an empty and closed channel"),
        }
    }
}
