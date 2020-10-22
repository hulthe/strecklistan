use crate::app::Msg;
use crate::generated::css_classes::C;
use seed::app::cmds::timeout;
use seed::prelude::*;
use seed::*;
use std::collections::BTreeMap;

pub type NotificationId = u32;

#[derive(Debug, Clone)]
pub enum NotificationMessage {
    ShowNotification {
        duration_ms: u32,
        notification: Notification,
    },
    RemoveNotification(NotificationId),
}

#[derive(Default)]
pub struct NotificationManager {
    next_id: NotificationId,
    notifications: BTreeMap<NotificationId, Notification>,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub title: String,
    pub body: Option<String>,
}

impl NotificationManager {
    pub fn view(&self) -> Node<Msg> {
        div![
            class![C.notification_list],
            self.notifications.iter().map(|(id, notification)| {
                div![
                    class![C.notification],
                    p![class![C.notification_title], &notification.title],
                    if let Some(body) = &notification.body {
                        p![class![C.notification_body], &body]
                    } else {
                        empty![]
                    },
                    simple_ev(
                        Ev::Click,
                        Msg::NotificationMessage(NotificationMessage::RemoveNotification(*id))
                    ),
                ]
            })
        ]
    }

    pub fn update(&mut self, msg: NotificationMessage, orders: &mut impl Orders<Msg>) {
        match msg {
            NotificationMessage::ShowNotification {
                duration_ms,
                notification,
            } => {
                let id = self.next_id;
                self.next_id += 1;

                self.notifications.insert(id, notification);
                orders.perform_cmd(timeout(duration_ms, move || {
                    Msg::NotificationMessage(NotificationMessage::RemoveNotification(id))
                }));
            }
            NotificationMessage::RemoveNotification(id) => {
                self.notifications.remove(&id);
            }
        }
    }
}
