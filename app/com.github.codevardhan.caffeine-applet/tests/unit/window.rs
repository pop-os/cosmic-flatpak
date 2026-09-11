//! Unit tests for `src/window.rs`.
//!
//! Loaded by the `#[path]` declaration at the bottom of that file, so
//! `use super::*` gives access to private items like `CaffeineDuration`,
//! `acquire_inhibit`, and the private fields of `CaffeineApplet`.
//!
//! Run with: cargo test
//! Ignored (logind-dependent) tests: cargo test -- --include-ignored

use super::*;
use std::fs::File;

/// A stand-in inhibit fd that doesn't touch D-Bus/logind at all.
/// Lets us test `update()` state transitions for an "already active"
/// applet without a real system bus / logind session.
fn dummy_fd() -> OwnedFd {
    File::open("/dev/null")
        .expect("opening /dev/null should always succeed")
        .into()
}

//  CaffeineDuration::label ─

#[test]
fn label_minutes15() {
    assert_eq!(CaffeineDuration::Minutes15.label(), "15 minutes");
}

#[test]
fn label_minutes30() {
    assert_eq!(CaffeineDuration::Minutes30.label(), "30 minutes");
}

#[test]
fn label_hour1() {
    assert_eq!(CaffeineDuration::Hour1.label(), "1 hour");
}

#[test]
fn label_indefinite() {
    assert_eq!(CaffeineDuration::Indefinite.label(), "Indefinitely");
}

#[test]
fn labels_are_unique() {
    let mut labels: Vec<&str> = CaffeineDuration::ALL.iter().map(|c| c.label()).collect();
    let original_len = labels.len();
    labels.sort_unstable();
    labels.dedup();
    assert_eq!(labels.len(), original_len, "duplicate label found");
}

//  CaffeineDuration::duration ─

#[test]
fn duration_minutes15_is_15_minutes() {
    assert_eq!(
        CaffeineDuration::Minutes15.duration(),
        Some(Duration::from_secs(15 * 60))
    );
}

#[test]
fn duration_minutes30_is_30_minutes() {
    assert_eq!(
        CaffeineDuration::Minutes30.duration(),
        Some(Duration::from_secs(30 * 60))
    );
}

#[test]
fn duration_hour1_is_60_minutes() {
    assert_eq!(
        CaffeineDuration::Hour1.duration(),
        Some(Duration::from_secs(60 * 60))
    );
}

#[test]
fn duration_indefinite_is_none() {
    assert_eq!(CaffeineDuration::Indefinite.duration(), None);
}

#[test]
fn only_indefinite_has_no_duration() {
    for choice in CaffeineDuration::ALL {
        let has_duration = choice.duration().is_some();
        assert_eq!(
            has_duration,
            choice != CaffeineDuration::Indefinite,
            "{choice:?} duration()-presence disagrees with Indefinite check"
        );
    }
}

#[test]
fn timed_durations_increase_with_choice_order() {
    let m15 = CaffeineDuration::Minutes15.duration().unwrap();
    let m30 = CaffeineDuration::Minutes30.duration().unwrap();
    let h1 = CaffeineDuration::Hour1.duration().unwrap();
    assert!(m15 < m30, "15 minutes should be shorter than 30 minutes");
    assert!(m30 < h1, "30 minutes should be shorter than 1 hour");
}

//  CaffeineDuration::ALL ─

#[test]
fn all_contains_four_unique_variants() {
    let all = CaffeineDuration::ALL;
    assert_eq!(all.len(), 4);
    for (i, a) in all.iter().enumerate() {
        for (j, b) in all.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "ALL contains a duplicate variant");
            }
        }
    }
}

#[test]
fn all_contains_every_expected_variant() {
    let all = CaffeineDuration::ALL;
    assert!(all.contains(&CaffeineDuration::Minutes15));
    assert!(all.contains(&CaffeineDuration::Minutes30));
    assert!(all.contains(&CaffeineDuration::Hour1));
    assert!(all.contains(&CaffeineDuration::Indefinite));
}

//  CaffeineApplet::default ─

#[test]
fn default_applet_has_no_active_session() {
    let app = CaffeineApplet::default();
    assert!(app.popup.is_none());
    assert!(app.inhibit_fd.is_none());
    assert_eq!(app.active_choice, None);
    assert!(app.deadline.is_none());
}

//  update(): Message::PopupClosed ─

#[test]
fn popup_closed_clears_matching_id() {
    let mut app = CaffeineApplet::default();
    let id = Id::unique();
    app.popup = Some(id);

    cosmic::Application::update(&mut app, Message::PopupClosed(id));

    assert!(app.popup.is_none());
}

#[test]
fn popup_closed_ignores_mismatched_id() {
    let mut app = CaffeineApplet::default();
    let open_id = Id::unique();
    let other_id = Id::unique();
    app.popup = Some(open_id);

    cosmic::Application::update(&mut app, Message::PopupClosed(other_id));

    assert_eq!(app.popup, Some(open_id));
}

#[test]
fn popup_closed_on_already_closed_popup_is_noop() {
    let mut app = CaffeineApplet::default();
    let id = Id::unique();

    cosmic::Application::update(&mut app, Message::PopupClosed(id));

    assert!(app.popup.is_none());
}

//  update(): Message::Disable ─

#[test]
fn disable_clears_active_session() {
    let mut app = CaffeineApplet::default();
    app.inhibit_fd = Some(dummy_fd());
    app.active_choice = Some(CaffeineDuration::Hour1);
    app.deadline = Some(Instant::now() + Duration::from_secs(60));

    cosmic::Application::update(&mut app, Message::Disable);

    assert!(app.inhibit_fd.is_none());
    assert_eq!(app.active_choice, None);
    assert!(app.deadline.is_none());
}

#[test]
fn disable_closes_open_popup() {
    let mut app = CaffeineApplet::default();
    app.inhibit_fd = Some(dummy_fd());
    app.popup = Some(Id::unique());

    cosmic::Application::update(&mut app, Message::Disable);

    assert!(app.popup.is_none());
}

#[test]
fn disable_when_already_inactive_is_noop() {
    let mut app = CaffeineApplet::default();

    cosmic::Application::update(&mut app, Message::Disable);

    assert!(app.inhibit_fd.is_none());
    assert_eq!(app.active_choice, None);
    assert!(app.deadline.is_none());
}

//  update(): Message::Enable, already-active fd (no D-Bus needed) ─

#[test]
fn enable_when_already_active_updates_choice_without_reacquiring() {
    let mut app = CaffeineApplet::default();
    app.inhibit_fd = Some(dummy_fd());
    app.active_choice = Some(CaffeineDuration::Minutes15);
    app.deadline = Some(Instant::now() + Duration::from_secs(15 * 60));

    cosmic::Application::update(&mut app, Message::Enable(CaffeineDuration::Hour1));

    assert!(
        app.inhibit_fd.is_some(),
        "existing fd should be kept, not dropped"
    );
    assert_eq!(app.active_choice, Some(CaffeineDuration::Hour1));
    assert!(app.deadline.is_some());
}

#[test]
fn enable_indefinite_when_already_active_clears_deadline() {
    let mut app = CaffeineApplet::default();
    app.inhibit_fd = Some(dummy_fd());
    app.active_choice = Some(CaffeineDuration::Minutes15);
    app.deadline = Some(Instant::now() + Duration::from_secs(15 * 60));

    cosmic::Application::update(&mut app, Message::Enable(CaffeineDuration::Indefinite));

    assert_eq!(app.active_choice, Some(CaffeineDuration::Indefinite));
    assert!(app.deadline.is_none());
}

#[test]
fn enable_closes_popup_when_already_active() {
    let mut app = CaffeineApplet::default();
    app.inhibit_fd = Some(dummy_fd());
    app.popup = Some(Id::unique());

    cosmic::Application::update(&mut app, Message::Enable(CaffeineDuration::Minutes30));

    assert!(app.popup.is_none());
}

//  update(): Message::Tick ─

#[test]
fn tick_before_deadline_leaves_session_active() {
    let mut app = CaffeineApplet::default();
    app.inhibit_fd = Some(dummy_fd());
    app.active_choice = Some(CaffeineDuration::Minutes15);
    app.deadline = Some(Instant::now() + Duration::from_secs(60));

    cosmic::Application::update(&mut app, Message::Tick);

    assert!(app.inhibit_fd.is_some());
    assert_eq!(app.active_choice, Some(CaffeineDuration::Minutes15));
    assert!(app.deadline.is_some());
}

#[test]
fn tick_after_deadline_clears_session() {
    let mut app = CaffeineApplet::default();
    app.inhibit_fd = Some(dummy_fd());
    app.active_choice = Some(CaffeineDuration::Minutes15);
    // Deadline already in the past.
    app.deadline = Some(Instant::now() - Duration::from_secs(1));

    cosmic::Application::update(&mut app, Message::Tick);

    assert!(app.inhibit_fd.is_none());
    assert_eq!(app.active_choice, None);
    assert!(app.deadline.is_none());
}

#[test]
fn tick_with_no_deadline_is_noop() {
    let mut app = CaffeineApplet::default();

    cosmic::Application::update(&mut app, Message::Tick);

    assert!(app.inhibit_fd.is_none());
    assert_eq!(app.active_choice, None);
    assert!(app.deadline.is_none());
}

//  Message ─

#[test]
fn message_variants_are_debug_printable() {
    // Sanity check only: make sure Debug is derivable/doesn't panic
    // for the variants that are trivial to construct in a unit test.
    assert_eq!(format!("{:?}", Message::Disable), "Disable");
    assert_eq!(format!("{:?}", Message::Tick), "Tick");
    assert!(format!("{:?}", Message::Enable(CaffeineDuration::Hour1)).contains("Hour1"));
}

//  acquire_inhibit (logind required) ─

/// Requires a running `org.freedesktop.login1` on the system bus.
/// Run with: cargo test -- --include-ignored
#[test]
#[ignore = "requires org.freedesktop.login1 on the system bus"]
fn acquire_inhibit_returns_valid_fd() {
    let fd = acquire_inhibit().expect("should acquire inhibit fd");
    drop(fd);
}

#[test]
#[ignore = "requires org.freedesktop.login1 on the system bus"]
fn inhibit_fd_is_open_while_held_and_released_on_drop() {
    let fd = acquire_inhibit().expect("acquire");
    assert!(fd.try_clone().is_ok(), "fd should be open while held");
    drop(fd);
    // Kernel guarantees the inhibit lock is released when the fd is closed.
}

#[test]
#[ignore = "requires org.freedesktop.login1 on the system bus"]
fn enable_from_scratch_acquires_inhibit_and_sets_deadline() {
    let mut app = CaffeineApplet::default();

    cosmic::Application::update(&mut app, Message::Enable(CaffeineDuration::Minutes15));

    assert!(app.inhibit_fd.is_some());
    assert_eq!(app.active_choice, Some(CaffeineDuration::Minutes15));
    assert!(app.deadline.is_some());
}
