use super::BrowserSessionCommand;
use katana_render_runtime::HtmlBrowserInput;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CoalescingKey {
    PointerMove,
    Scroll,
    Resize,
    Refresh,
}

pub(super) fn enqueue_command(
    pending: &mut VecDeque<BrowserSessionCommand>,
    command: BrowserSessionCommand,
) {
    let Some(next_key) = coalescing_key(&command) else {
        pending.push_back(command);
        return;
    };
    let current = pending
        .iter_mut()
        .rev()
        .take_while(|current| coalescing_key(current).is_some())
        .find(|current| coalescing_key(current) == Some(next_key));
    let Some(current) = current else {
        pending.push_back(command);
        return;
    };
    let merged = merge_command(current, command);
    debug_assert!(merged.is_ok(), "matching mailbox commands must merge");
}

fn coalescing_key(command: &BrowserSessionCommand) -> Option<CoalescingKey> {
    match command {
        BrowserSessionCommand::Input(HtmlBrowserInput::PointerMove { .. }) => {
            Some(CoalescingKey::PointerMove)
        }
        BrowserSessionCommand::Input(HtmlBrowserInput::Scroll { .. }) => {
            Some(CoalescingKey::Scroll)
        }
        BrowserSessionCommand::Resize(_) => Some(CoalescingKey::Resize),
        BrowserSessionCommand::Refresh => Some(CoalescingKey::Refresh),
        BrowserSessionCommand::Input(_)
        | BrowserSessionCommand::Navigate(_)
        | BrowserSessionCommand::Close => None,
    }
}

pub(super) fn merge_command(
    command: &mut BrowserSessionCommand,
    next: BrowserSessionCommand,
) -> Result<(), BrowserSessionCommand> {
    match (command, next) {
        (BrowserSessionCommand::Input(command), BrowserSessionCommand::Input(next)) => {
            merge_input(command, next).map_err(BrowserSessionCommand::Input)
        }
        (command @ BrowserSessionCommand::Resize(_), BrowserSessionCommand::Resize(viewport)) => {
            *command = BrowserSessionCommand::Resize(viewport);
            Ok(())
        }
        (BrowserSessionCommand::Refresh, BrowserSessionCommand::Refresh) => Ok(()),
        (_, next) => Err(next),
    }
}

fn merge_input(
    command: &mut HtmlBrowserInput,
    next: HtmlBrowserInput,
) -> Result<(), HtmlBrowserInput> {
    match (command, next) {
        (
            HtmlBrowserInput::Scroll { delta_x, delta_y },
            HtmlBrowserInput::Scroll {
                delta_x: next_x,
                delta_y: next_y,
            },
        ) => {
            *delta_x += next_x;
            *delta_y += next_y;
            Ok(())
        }
        (
            HtmlBrowserInput::PointerMove { x, y },
            HtmlBrowserInput::PointerMove {
                x: next_x,
                y: next_y,
            },
        ) => {
            *x = next_x;
            *y = next_y;
            Ok(())
        }
        (_, next) => Err(next),
    }
}
