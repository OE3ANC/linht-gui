use std::io::{self, Read};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::thread;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    NextFlowgraph,
    PreviousFlowgraph,
    Run,
    Stop,
    Quit,
}

/// Handles keyboard input from both stdio and keypad simultaneously
pub struct InputHandler {
    receiver: Receiver<InputEvent>,
}

impl InputHandler {
    /// Creates a new input handler that uses both stdio and keypad (if available)
    pub fn new() -> Self {
        let (tx, rx) = channel();
        
        // Always start stdio input thread
        let tx_stdio = tx.clone();
        thread::spawn(move || {
            let stdin = io::stdin();
            let mut buffer = [0; 1];
            
            loop {
                if stdin.lock().read_exact(&mut buffer).is_ok() {
                    let event = match buffer[0] {
                        b'+' => Some(InputEvent::NextFlowgraph),
                        b'-' => Some(InputEvent::PreviousFlowgraph),
                        b'r' | b'R' => Some(InputEvent::Run),
                        b's' | b'S' => Some(InputEvent::Stop),
                        b'q' | b'Q' => Some(InputEvent::Quit),
                        _ => None,
                    };
                    
                    if let Some(event) = event {
                        if tx_stdio.send(event.clone()).is_err() {
                            break;
                        }
                        
                        if event == InputEvent::Quit {
                            break;
                        }
                    }
                }
            }
        });
        
        // Try to also start keypad input if available
        if let Some(keypad_path) = find_keypad_device() {
            println!("[Input] Found keypad at {}", keypad_path.display());
            if let Err(e) = Self::start_keypad_thread(keypad_path, tx.clone()) {
                println!("[Input] Failed to open keypad: {}", e);
            } else {
                println!("[Input] Keypad enabled");
            }
        }
        
        println!("[Input] Stdio input active");
        InputHandler { receiver: rx }
    }
    
    /// Starts the keypad input thread
    fn start_keypad_thread(device_path: PathBuf, tx: std::sync::mpsc::Sender<InputEvent>) -> Result<(), Box<dyn std::error::Error>> {
        use evdev::{Device, InputEventKind, Key};
        
        let mut device = Device::open(&device_path)?;
        
        thread::spawn(move || {
            loop {
                if let Ok(events) = device.fetch_events() {
                    for event in events {
                        if let InputEventKind::Key(key) = event.kind() {
                            if event.value() == 1 {
                                let input_event = match key {
                                    Key::KEY_UP => Some(InputEvent::NextFlowgraph),
                                    Key::KEY_DOWN => Some(InputEvent::PreviousFlowgraph),
                                    Key::KEY_ENTER => Some(InputEvent::Run),
                                    Key::KEY_ESC => Some(InputEvent::Stop),
                                    _ => None,
                                };
                                
                                if let Some(event) = input_event {
                                    if tx.send(event).is_err() {
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Checks for available input without blocking
    pub fn check_input(&self) -> Option<InputEvent> {
        match self.receiver.try_recv() {
            Ok(event) => Some(event),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(InputEvent::Quit),
        }
    }
}

/// Find the keypad device by scanning /sys/class/input/*/name
fn find_keypad_device() -> Option<PathBuf> {
    let input_dir = PathBuf::from("/sys/class/input");
    
    if let Ok(entries) = fs::read_dir(&input_dir) {
        for entry in entries.flatten() {
            let name_path = entry.path().join("name");
            if let Ok(name) = fs::read_to_string(&name_path) {
                if name.trim() == "matrix-keypad" {
                    if let Some(input_name) = entry.file_name().to_str() {
                        if let Some(num) = input_name.strip_prefix("input") {
                            let event_path = PathBuf::from(format!("/dev/input/event{}", num));
                            if event_path.exists() {
                                return Some(event_path);
                            }
                        }
                    }
                }
            }
        }
    }
    
    None
}