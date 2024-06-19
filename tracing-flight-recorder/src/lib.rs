//! An Experiment subscriber for continuous tracing for tokio
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/tokio-rs/tracing/master/assets/logo-type.png",
    html_favicon_url = "https://raw.githubusercontent.com/tokio-rs/tracing/master/assets/favicon.ico",
    issue_tracker_base_url = "https://github.com/tokio-rs/tracing/issues/"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_interfaces,
    private_bounds,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use std::{
    cell::RefCell,
    io::Write,
    sync::{
        atomic::AtomicBool,
        mpsc::{channel, Receiver, Sender},
        RwLock,
    },
};
use tracing::{event, field::Visit, Subscriber};
use tracing_subscriber::registry::LookupSpan;

pub(crate) mod ring_buffer;

#[derive(Debug)]
struct ThreadLocalBuffer {
    sender: std::sync::mpsc::Sender<Box<[u8]>>,
    bytes: [u8; DEFAULT_BUF_SIZE],
    cur: usize,
}

const DEFAULT_BUF_SIZE: usize = 100000;

static INITIALIZED: AtomicBool = AtomicBool::new(false);
static SENDER: RwLock<Option<Sender<Box<[u8]>>>> = RwLock::new(None);

thread_local! {
    static TLS_BUF: RefCell<ThreadLocalBuffer> =  RefCell::new(ThreadLocalBuffer::new())
}

impl ThreadLocalBuffer {
    fn new() -> Self {
        let mut sender = SENDER.write().unwrap();
        if !INITIALIZED.fetch_or(true, std::sync::atomic::Ordering::SeqCst) {
            let (tx, rx) = channel::<Box<[u8]>>();

            std::thread::spawn(move || run_recv_thread(rx));
            // sendのset
            *sender = Some(tx);
        }
        let sender = sender
            .as_ref()
            .expect("sender must be initialized.")
            .clone();

        Self {
            sender: sender,
            bytes: [0; DEFAULT_BUF_SIZE],
            cur: 0,
        }
    }
}

fn run_recv_thread(rx: Receiver<Box<[u8]>>) {
    let mut i = 0;
    while let Ok(b) = rx.recv() {
        println!("{i}: get {:?}, sum: {:?}", &b.len(), (&b.len() * i));
        i += 1;
    }
}

impl std::io::Write for ThreadLocalBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut written = 0;

        // println!(
        //     "cur: {:?}, buf: {:?}, bytes.len: {:?}",
        //     self.cur,
        //     buf,
        //     self.bytes.len(),
        // );

        while written < buf.len() {
            let remaining = DEFAULT_BUF_SIZE - self.cur;
            let to_write = std::cmp::min(remaining, buf.len() - written);

            self.bytes[self.cur..self.cur + to_write]
                .copy_from_slice(&buf[written..written + to_write]);
            self.cur += to_write;
            written += to_write;

            if DEFAULT_BUF_SIZE <= self.cur {
                self.flush().unwrap();
                self.cur = 0;
            }
        }

        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let b = Box::new(self.bytes);
        self.sender.send(b).unwrap();

        Ok(())
    }
}

impl Visit for ThreadLocalBuffer {
    fn record_debug(&mut self, _field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        // TODO: this seems a bit ugly. are there any other ways?
        let formatted_string = format!("{:?}", value);
        self.write(formatted_string.into_bytes().as_slice())
            .unwrap();
    }
}

/// docs
#[derive(Debug)]
pub struct Layer {}

impl Layer {
    /// docs
    pub fn builder() -> LayerBuilder {
        LayerBuilder::new()
    }

    /// docs
    pub fn new() -> Self {
        LayerBuilder::default().build()
    }
}

impl<S> tracing_subscriber::Layer<S> for Layer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &event::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        // write to thread local buffer
        TLS_BUF.with_borrow_mut(|tls_buf| {
            event.record(tls_buf);
        });
    }
}

/// docs
#[derive(Debug)]
pub struct LayerBuilder {}

impl LayerBuilder {
    /// docs
    pub fn new() -> Self {
        Self::default()
    }

    fn build(self) -> Layer {
        Layer {}
    }
}

impl Default for LayerBuilder {
    fn default() -> Self {
        Self {}
    }
}
