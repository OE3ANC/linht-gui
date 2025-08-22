use std::fs;
use std::path::PathBuf;
use std::process::{Command, Child, Stdio};
use std::io::{BufReader, BufRead};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

mod framebuffer;
mod display;
mod m17;
mod input;

use framebuffer::{Framebuffer, FontId};
use display::Display;
use m17::parse_m17_line;
use input::{InputEvent, InputHandler};

struct AppState {
    flowgraphs: Vec<PathBuf>,
    current_index: usize,
    running_process: Option<Child>,
    process_output_rx: Option<Receiver<String>>,
    last_message: Option<String>,
    display: Display,
    input_handler: InputHandler,
}

impl AppState {
    fn new(framebuffer_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut fb = Framebuffer::new(framebuffer_path)?;
        fb.load_font(FontId::Regular, Some("fonts/DidactGothic-Regular.ttf"))?;
        
        let flowgraphs = discover_flowgraphs("flowgraphs")?;
        if flowgraphs.is_empty() {
            return Err("No flowgraphs found in flowgraphs/ directory".into());
        }
        
        println!("[Main] Found {} flowgraphs", flowgraphs.len());
        for (i, fg) in flowgraphs.iter().enumerate() {
            println!("[Main]   [{}] {}", i, fg.file_name().unwrap_or_default().to_string_lossy());
        }
        
        Ok(AppState {
            flowgraphs,
            current_index: 0,
            running_process: None,
            process_output_rx: None,
            last_message: None,
            display: Display::new(fb),
            input_handler: InputHandler::new(),
        })
    }
    
    fn current_flowgraph_name(&self) -> String {
        self.flowgraphs[self.current_index]
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }
    
    fn next_flowgraph(&mut self) {
        if self.flowgraphs.is_empty() {
            return;
        }
        self.current_index = (self.current_index + 1) % self.flowgraphs.len();
        println!("[Main] Switched to flowgraph: {}", self.current_flowgraph_name());
    }
    
    fn previous_flowgraph(&mut self) {
        if self.flowgraphs.is_empty() {
            return;
        }
        if self.current_index == 0 {
            self.current_index = self.flowgraphs.len() - 1;
        } else {
            self.current_index -= 1;
        }
        println!("[Main] Switched to flowgraph: {}", self.current_flowgraph_name());
    }
    
    fn start_flowgraph(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.stop_flowgraph();
        
        let flowgraph_path = &self.flowgraphs[self.current_index];
        println!("[Main] Starting flowgraph: {}", flowgraph_path.display());
        
        let mut child = Command::new("python3")
            .arg(flowgraph_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .env("PYTHONUNBUFFERED", "1")
            .spawn()?;
        
        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let (tx, rx) = channel();
        
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if line.contains("LSF_CRC_OK") {
                        if tx.send(line).is_err() {
                            break;
                        }
                    }
                }
            }
        });
        
        self.running_process = Some(child);
        self.process_output_rx = Some(rx);
        
        println!("[Main] Flowgraph started");
        Ok(())
    }
    
    fn stop_flowgraph(&mut self) {
        if let Some(mut child) = self.running_process.take() {
            println!("[Main] Stopping flowgraph...");
            let _ = child.kill();
            let _ = child.wait();
        }
        self.process_output_rx = None;
        self.last_message = None;
    }
    
    fn check_process_output(&mut self) {
        if let Some(rx) = &self.process_output_rx {
            match rx.try_recv() {
                Ok(line) => {
                    if let Some(message) = parse_m17_line(&line) {
                        println!("[M17] Message: {}", message);
                        self.display.show_m17_message(&message);
                        self.last_message = Some(message);
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    println!("[Main] Process output ended");
                    self.running_process = None;
                    self.process_output_rx = None;
                }
                Err(TryRecvError::Empty) => {
                }
            }
        }
        
        if let Some(ref mut child) = self.running_process {
            match child.try_wait() {
                Ok(Some(status)) => {
                    println!("[Main] Process exited with: {:?}", status);
                    self.running_process = None;
                    self.process_output_rx = None;
                }
                Ok(None) => {
                }
                Err(e) => {
                    println!("[Main] Error checking process status: {}", e);
                    self.running_process = None;
                    self.process_output_rx = None;
                }
            }
        }
    }
    
    fn is_running(&self) -> bool {
        self.running_process.is_some()
    }
    
    fn update_display(&mut self) {
        if self.is_running() {
            self.display.show_status(&format!("Running: {}", self.current_flowgraph_name()));
        } else {
            self.display.show_idle(&self.current_flowgraph_name());
        }
    }
}

fn discover_flowgraphs(dir: &str) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut flowgraphs = Vec::new();
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("py") {
            flowgraphs.push(path);
        }
    }
    
    flowgraphs.sort();
    Ok(flowgraphs)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[Main] Starting...");
    println!("[Main] Commands: + (next), - (previous), r (run), s (stop), q (quit)");
    
    let mut state = AppState::new("/dev/fb0")?;
    
    state.display.show_welcome();
    thread::sleep(Duration::from_secs(2));
    
    let mut last_display_update = Instant::now();
    
    loop {
        if let Some(event) = state.input_handler.check_input() {
            match event {
                InputEvent::NextFlowgraph => {
                    if !state.is_running() {
                        state.next_flowgraph();
                        state.update_display();
                    }
                }
                InputEvent::PreviousFlowgraph => {
                    if !state.is_running() {
                        state.previous_flowgraph();
                        state.update_display();
                    }
                }
                InputEvent::Run => {
                    if !state.is_running() {
                        if let Err(e) = state.start_flowgraph() {
                            println!("[Main] Failed to start flowgraph: {}", e);
                        }
                        state.update_display();
                    }
                }
                InputEvent::Stop => {
                    if state.is_running() {
                        state.stop_flowgraph();
                        state.update_display();
                    }
                }
                InputEvent::Quit => {
                    println!("[Main] Exiting...");
                    state.stop_flowgraph();
                    break;
                }
            }
        }
        
        state.check_process_output();
        
        if last_display_update.elapsed() > Duration::from_secs(1) {
            state.update_display();
            last_display_update = Instant::now();
        }
        
        thread::sleep(Duration::from_millis(50));
    }
    
    state.display.show_shutdown();
    
    Ok(())
}