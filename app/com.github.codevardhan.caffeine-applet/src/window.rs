use std::os::fd::OwnedFd;
use std::time::{Duration, Instant};

use cosmic::app::Core;
use cosmic::iced::window::Id;
use cosmic::iced::{Rectangle, Subscription, Task};
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget::{list_column, text};
use cosmic::Element;
use zbus::blocking::Connection;
use zbus::zvariant::OwnedFd as ZbusFd;

const ID: &str = "com.github.codevardhan.caffeine-applet";
const ON: &str = "com.github.codevardhan.caffeine-applet.On";
const OFF: &str = "com.github.codevardhan.caffeine-applet.Off";

/// How long a caffeine session should stay active for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaffeineDuration {
    Minutes15,
    Minutes30,
    Hour1,
    Indefinite,
}

impl CaffeineDuration {
    const ALL: [CaffeineDuration; 4] = [
        Self::Minutes15,
        Self::Minutes30,
        Self::Hour1,
        Self::Indefinite,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Minutes15 => "15 minutes",
            Self::Minutes30 => "30 minutes",
            Self::Hour1 => "1 hour",
            Self::Indefinite => "Indefinitely",
        }
    }

    /// `None` means the session never expires on its own.
    fn duration(self) -> Option<Duration> {
        match self {
            Self::Minutes15 => Some(Duration::from_secs(15 * 60)),
            Self::Minutes30 => Some(Duration::from_secs(30 * 60)),
            Self::Hour1 => Some(Duration::from_secs(60 * 60)),
            Self::Indefinite => None,
        }
    }
}

#[derive(Default)]
pub struct CaffeineApplet {
    core: Core,
    popup: Option<Id>,
    inhibit_fd: Option<OwnedFd>,
    active_choice: Option<CaffeineDuration>,
    deadline: Option<Instant>,
}

#[derive(Clone, Debug)]
pub enum Message {
    PopupClosed(Id),
    Enable(CaffeineDuration),
    Disable,
    Tick,
    Surface(cosmic::surface::Action),
}

/// Ask logind for an idle+sleep inhibit lock.
/// Returns an OwnedFd — the inhibit stays active as long as this fd is open.
fn acquire_inhibit() -> Result<OwnedFd, Box<dyn std::error::Error>> {
    let conn = Connection::system()?;
    let reply: ZbusFd = conn
        .call_method(
            Some("org.freedesktop.login1"),
            "/org/freedesktop/login1",
            Some("org.freedesktop.login1.Manager"),
            "Inhibit",
            &(
                "idle:sleep",
                "Caffeine Applet",
                "Caffeine session active",
                "block",
            ),
        )?
        .body()
        .deserialize()?;

    Ok(reply.into())
}

impl cosmic::Application for CaffeineApplet {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(
        core: Core,
        _flags: Self::Flags,
    ) -> (Self, cosmic::Task<cosmic::Action<Self::Message>>) {
        let window = CaffeineApplet {
            core,
            ..Default::default()
        };
        (window, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> cosmic::Task<cosmic::Action<Self::Message>> {
        match message {
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
            Message::Enable(choice) => {
                if self.inhibit_fd.is_none() {
                    match acquire_inhibit() {
                        Ok(fd) => self.inhibit_fd = Some(fd),
                        Err(err) => {
                            eprintln!(
                                "Failed to acquire inhibit lock (is logind/elogind running?): {err}"
                            );
                            return Task::none();
                        }
                    }
                }
                self.active_choice = Some(choice);
                self.deadline = choice.duration().map(|d| Instant::now() + d);

                if let Some(id) = self.popup.take() {
                    return cosmic::task::message(cosmic::Action::Cosmic(
                        cosmic::app::Action::Surface(destroy_popup(id)),
                    ));
                }
            }
            Message::Disable => {
                // Drop the fd → releases the inhibit lock
                self.inhibit_fd = None;
                self.active_choice = None;
                self.deadline = None;

                if let Some(id) = self.popup.take() {
                    return cosmic::task::message(cosmic::Action::Cosmic(
                        cosmic::app::Action::Surface(destroy_popup(id)),
                    ));
                }
            }
            Message::Tick => {
                if let Some(deadline) = self.deadline {
                    if Instant::now() >= deadline {
                        self.inhibit_fd = None;
                        self.active_choice = None;
                        self.deadline = None;
                    }
                }
            }
            Message::Surface(a) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(a),
                ));
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        if self.deadline.is_some() {
            cosmic::iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let icon = if self.inhibit_fd.is_some() { ON } else { OFF };
        let have_popup = self.popup;

        self.core
            .applet
            .icon_button(icon)
            .on_press_with_rectangle(move |offset, bounds| {
                if let Some(id) = have_popup {
                    Message::Surface(destroy_popup(id))
                } else {
                    Message::Surface(app_popup::<CaffeineApplet>(
                        move |state: &mut CaffeineApplet| {
                            let new_id = Id::unique();
                            state.popup = Some(new_id);
                            let mut popup_settings = state.core.applet.get_popup_settings(
                                state.core.main_window_id().unwrap(),
                                new_id,
                                None,
                                None,
                                None,
                            );

                            popup_settings.positioner.anchor_rect = Rectangle {
                                x: (bounds.x - offset.x) as i32,
                                y: (bounds.y - offset.y) as i32,
                                width: bounds.width as i32,
                                height: bounds.height as i32,
                            };

                            popup_settings
                        },
                        Some(Box::new(move |state: &CaffeineApplet| {
                            Element::from(state.core.applet.popup_container(state.menu_content()))
                                .map(cosmic::Action::App)
                        })),
                    ))
                }
            })
            .into()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

impl CaffeineApplet {
    fn menu_content(&self) -> Element<'_, Message> {
        let mut list = list_column().padding(5).spacing(4);

        for choice in CaffeineDuration::ALL {
            let label = if self.active_choice == Some(choice) {
                format!("✓  {}", choice.label())
            } else {
                choice.label().to_string()
            };
            list = list
                .add(cosmic::applet::menu_button(text(label)).on_press(Message::Enable(choice)));
        }

        if self.inhibit_fd.is_some() {
            list =
                list.add(cosmic::applet::menu_button(text("Turn off")).on_press(Message::Disable));
        }

        list.into()
    }
}

// Tests live in tests/unit/window.rs.
// #[path] lets Rust load them from there while keeping access to the
// private functions/fields above via `use super::*`.
#[cfg(test)]
#[path = "../tests/unit/window.rs"]
mod tests;