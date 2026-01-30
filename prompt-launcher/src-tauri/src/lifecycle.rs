use std::time::{Duration, Instant};

pub const TOGGLE_DEBOUNCE: Duration = Duration::from_millis(300);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateDecision {
    Proceed,
    Debounced,
    DeferShow,
}

#[derive(Debug)]
pub struct LauncherGate {
    ui_ready: bool,
    pending_show: bool,
    last_toggle_at: Option<Instant>,
}

impl LauncherGate {
    pub fn new() -> Self {
        Self {
            ui_ready: false,
            pending_show: false,
            last_toggle_at: None,
        }
    }

    pub fn request_show(&mut self) -> bool {
        if self.ui_ready {
            return true;
        }
        self.pending_show = true;
        false
    }

    pub fn set_ui_ready(&mut self) -> bool {
        self.ui_ready = true;
        let pending = self.pending_show;
        self.pending_show = false;
        pending
    }

    pub fn allow_toggle(&mut self, now: Instant) -> GateDecision {
        if !self.ui_ready {
            self.pending_show = true;
            return GateDecision::DeferShow;
        }
        if let Some(last) = self.last_toggle_at {
            if now.duration_since(last) < TOGGLE_DEBOUNCE {
                return GateDecision::Debounced;
            }
        }
        self.last_toggle_at = Some(now);
        GateDecision::Proceed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_show_defers_until_ready() {
        let mut gate = LauncherGate::new();
        assert!(!gate.request_show());
        assert!(gate.set_ui_ready());
    }

    #[test]
    fn request_show_immediate_when_ready() {
        let mut gate = LauncherGate::new();
        gate.set_ui_ready();
        assert!(gate.request_show());
    }

    #[test]
    fn allow_toggle_debounces() {
        let mut gate = LauncherGate::new();
        gate.set_ui_ready();
        let start = Instant::now();
        assert_eq!(gate.allow_toggle(start), GateDecision::Proceed);
        assert_eq!(
            gate.allow_toggle(start + Duration::from_millis(100)),
            GateDecision::Debounced
        );
        assert_eq!(
            gate.allow_toggle(start + Duration::from_millis(400)),
            GateDecision::Proceed
        );
    }

    #[test]
    fn allow_toggle_defers_before_ready() {
        let mut gate = LauncherGate::new();
        let start = Instant::now();
        assert_eq!(gate.allow_toggle(start), GateDecision::DeferShow);
        assert!(gate.set_ui_ready());
    }
}
