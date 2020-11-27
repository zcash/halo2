//! Helpers for modelling halo2 circuit performance.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use metrics::{Key, Recorder, Unit};

/// A [`metrics`] recorder for examining halo2 metrics.
///
/// # Examples
///
/// ```
/// use halo2::model::ModelRecorder;
///
/// let recorder = Box::leak(Box::new(ModelRecorder::default()));
/// metrics::set_recorder(recorder).unwrap();
///
/// // Create circuit, build and/or verify proof.
///
/// println!("{}", recorder);
/// recorder.clear();
///
/// // Perform another operation to collect separate metrics.
/// ```
#[derive(Debug)]
pub struct ModelRecorder {
    counters: Arc<RefCell<HashMap<Key, u64>>>,
}

impl Default for ModelRecorder {
    fn default() -> Self {
        ModelRecorder {
            counters: Default::default(),
        }
    }
}

impl fmt::Display for ModelRecorder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut counters = self
            .counters
            .try_borrow()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect::<Vec<_>>();

        counters.sort_by(|(k1, _), (k2, _)| {
            let key1 = (
                k1.name().to_string(),
                k1.labels()
                    .map(|l| (l.key(), l.value()))
                    .collect::<Vec<_>>(),
            );
            let key2 = (
                k2.name().to_string(),
                k2.labels()
                    .map(|l| (l.key(), l.value()))
                    .collect::<Vec<_>>(),
            );
            key1.cmp(&key2)
        });

        writeln!(f, "Recorded metrics:")?;
        for (key, value) in counters.iter() {
            writeln!(f, "- {}: {}", key, value)?;
        }
        Ok(())
    }
}

impl Recorder for ModelRecorder {
    fn register_counter(&self, _key: Key, _unit: Option<Unit>, _description: Option<&'static str>) {
    }

    fn register_gauge(&self, _key: Key, _unit: Option<Unit>, _description: Option<&'static str>) {}

    fn register_histogram(
        &self,
        _key: Key,
        _unit: Option<Unit>,
        _description: Option<&'static str>,
    ) {
    }

    fn increment_counter(&self, key: Key, value: u64) {
        *self
            .counters
            .try_borrow_mut()
            .unwrap()
            .entry(key)
            .or_default() += value;
    }

    fn update_gauge(&self, _key: Key, _value: f64) {
        unimplemented!()
    }

    fn record_histogram(&self, _key: Key, _value: u64) {
        unimplemented!()
    }
}

impl ModelRecorder {
    /// Clear all recorded metrics.
    pub fn clear(&self) {
        self.counters.try_borrow_mut().unwrap().clear();
    }
}
