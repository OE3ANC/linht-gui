use crate::framebuffer::{Framebuffer, Color, Point, FontId};
use std::sync::{Arc, Mutex};

/// Display manager for rendering UI elements to the framebuffer
pub struct Display {
    framebuffer: Arc<Mutex<Framebuffer>>,
}

impl Display {
    /// Creates a new Display instance with the given framebuffer
    pub fn new(framebuffer: Framebuffer) -> Self {
        Display {
            framebuffer: Arc::new(Mutex::new(framebuffer)),
        }
    }
    
    /// Shows the welcome screen with LinHT branding
    pub fn show_welcome(&mut self) {
        if let Ok(mut fb) = self.framebuffer.lock() {
            let _ = fb.clear_screen(Color::black());
            
            let _ = fb.write_text(
                ">LinHT_",
                Point::new(10, 65),
                38.0,
                Color::green(),
                FontId::Regular
            );
            
            let _ = fb.write_text(
                "by M17 Foundation",
                Point::new(20, 90),
                14.0,
                Color::white(),
                FontId::Regular
            );
            
            let _ = fb.flush();
        }
    }
    
    /// Shows the idle screen with frequencies and current flowgraph
    pub fn show_idle(&mut self, flowgraph_name: &str) {
        if let Ok(mut fb) = self.framebuffer.lock() {
            let _ = fb.clear_screen(Color::black());
            
            self.draw_status_bar(&mut fb, "Ready");
            
            let _ = fb.write_text(
                "RX: 438.300 MHz",
                Point::new(8, 40),
                16.0,
                Color::green(),
                FontId::Regular
            );
            
            let _ = fb.write_text(
                "TX: 430.700 MHz",
                Point::new(8, 60),
                16.0,
                Color::new(241, 196, 15, 255),
                FontId::Regular
            );
            
            let _ = fb.write_text(
                "Flowgraph:",
                Point::new(10, 90),
                10.0,
                Color::new(149, 165, 166, 255),
                FontId::Regular
            );
            
            let _ = fb.write_text(
                flowgraph_name,
                Point::new(10, 105),
                12.0,
                Color::white(),
                FontId::Regular
            );
            
            let _ = fb.flush();
        }
    }
    
    /// Updates only the status bar without clearing the screen
    pub fn show_status(&mut self, status: &str) {
        if let Ok(mut fb) = self.framebuffer.lock() {
            self.draw_status_bar(&mut fb, status);
            let _ = fb.flush();
        }
    }
    
    /// Displays an M17 protocol message with parsed fields
    pub fn show_m17_message(&mut self, message: &str) {
        if let Ok(mut fb) = self.framebuffer.lock() {
            for y in 17..128 {
                for x in 0..160 {
                    let _ = fb.set_pixel(x, y, Color::black());
                }
            }
            
            let _ = fb.write_text(
                "M17 Message",
                Point::new(10, 30),
                14.0,
                Color::green(),
                FontId::Regular
            );
            
            let lines: Vec<&str> = message.split('\n').collect();
            let mut y_pos = 50;
            
            for line in lines.iter().take(4) {
                if y_pos > 110 {
                    break;
                }
                
                let _ = fb.write_text(
                    line,
                    Point::new(10, y_pos),
                    10.0,
                    Color::white(),
                    FontId::Regular
                );
                
                y_pos += 15;
            }
            
            let _ = fb.flush();
        }
    }
    
    /// Clears the screen for shutdown
    pub fn show_shutdown(&mut self) {
        if let Ok(mut fb) = self.framebuffer.lock() {
            let _ = fb.clear_screen(Color::black());
            let _ = fb.flush();
        }
    }
    
    fn draw_status_bar(&self, fb: &mut Framebuffer, status: &str) {
        for y in 0..16 {
            for x in 0..160 {
                let _ = fb.set_pixel(x, y, Color::new(30, 30, 30, 255));
            }
        }
        
        let _ = fb.write_text(
            status,
            Point::new(5, 12),
            10.0,
            Color::white(),
            FontId::Regular
        );
        
        for x in 0..160 {
            let _ = fb.set_pixel(x, 16, Color::new(123, 123, 123, 255));
        }
    }
}