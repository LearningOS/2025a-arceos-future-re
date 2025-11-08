mod mem;
mod task;

cfg_fs! {
    mod fs;
    pub use fs::*;
}

cfg_net! {
    mod net;
    pub use net::*;
}

cfg_display! {
    mod display;
    pub use display::*;
}

mod stdio {
    use core::fmt;

    pub fn ax_console_read_byte() -> Option<u8> {
        axhal::console::getchar().map(|c| if c == b'\r' { b'\n' } else { c })
    }

    pub fn ax_console_write_bytes(buf: &[u8]) -> crate::AxResult<usize> {
        // Colorize lines that start with the special tag used by the exercise.
        const WITH_COLOR: &[u8] = b"[WithColor]";
        if buf.starts_with(WITH_COLOR) {
            axhal::console::write_bytes(b"\x1b[92;1m"); // bright green + bold
            axhal::console::write_bytes(buf);
            axhal::console::write_bytes(b"\x1b[0m"); // reset
            Ok(buf.len())
        } else {
            axhal::console::write_bytes(buf);
            Ok(buf.len())
        }
    }

    pub fn ax_console_write_fmt(args: fmt::Arguments) -> fmt::Result {
        axlog::print_fmt(args)
    }
}

mod time {
    pub use axhal::time::{
        monotonic_time as ax_monotonic_time, wall_time as ax_wall_time, TimeValue as AxTimeValue,
    };
}

mod rand {
    pub fn ax_random_u128() -> u128 {
        axhal::misc::random()
    }
}

pub use self::mem::*;
pub use self::stdio::*;
pub use self::task::*;
pub use self::time::*;
pub use self::rand::*;

pub use axhal::misc::terminate as ax_terminate;
pub use axio::PollState as AxPollState;
