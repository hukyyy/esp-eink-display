use esp_idf_svc::hal::gpio::{Gpio2, Level, PinDriver};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

pub enum LedProgram {
    Stable(Level),
    Blink(Duration),
    Shutdown,
}

pub struct InternalLed {
    thread_handle: Option<thread::JoinHandle<()>>,
    tx: Sender<LedProgram>,
}

impl<'a> InternalLed {
    pub fn new(gpio2: Gpio2) -> InternalLed {
        let (tx, rx) = mpsc::channel();
        let mut pin = PinDriver::output(gpio2).unwrap();
        let mut program = LedProgram::Stable(Level::High);

        let thread_handle = thread::spawn(move || loop {
            if let Ok(p) = rx.recv_timeout(Duration::from_millis(10)) {
                program = p;
            };

            match program {
                LedProgram::Stable(level) => {
                    pin.set_level(level).unwrap();
                    thread::sleep(Duration::from_millis(100));
                }
                LedProgram::Blink(duration) => {
                    pin.set_low().unwrap();
                    thread::sleep(duration);
                    pin.set_high().unwrap();
                    thread::sleep(duration - Duration::from_millis(10));
                }
                LedProgram::Shutdown => break,
            }
        });

        InternalLed {
            thread_handle: Some(thread_handle),
            tx,
        }
    }

    pub fn set_program(&mut self, program: LedProgram) {
        self.tx.send(program).unwrap();
    }
}

impl Drop for InternalLed {
    fn drop(&mut self) {
        self.tx.send(LedProgram::Shutdown).unwrap();
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}
