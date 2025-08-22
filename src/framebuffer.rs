use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use memmap2::{MmapMut, MmapOptions};
use fontdue::{Font, FontSettings};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontId {
    Regular,
}

impl FontId {
    pub fn default_path(&self) -> &'static str {
        match self {
            FontId::Regular => "fonts/DidactGothic-Regular.ttf",
        }
    }
}

pub const DISPLAY_WIDTH: u16 = 160;
pub const DISPLAY_HEIGHT: u16 = 128;
pub const BYTES_PER_PIXEL: usize = 4;
pub const FRAMEBUFFER_STRIDE: u16 = 160;
pub const FRAMEBUFFER_SIZE: usize = (FRAMEBUFFER_STRIDE as usize) * (DISPLAY_HEIGHT as usize) * BYTES_PER_PIXEL;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    
    #[inline]
    pub fn to_bytes(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
    
    pub const fn black() -> Self { Self::new(0, 0, 0, 255) }
    pub const fn white() -> Self { Self::new(255, 255, 255, 255) }
    pub const fn green() -> Self { Self::new(0, 255, 0, 255) }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

#[derive(Debug)]
pub enum FramebufferError {
    IoError(io::Error),
    InvalidCoordinate { x: u16, y: u16 },
    FontLoadError(String),
    FontNotLoaded(FontId),
    DeviceError(String),
    InvalidDeviceCapabilities(String),
    BufferSizeMismatch { expected: usize, actual: usize },
}

impl std::fmt::Display for FramebufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FramebufferError::IoError(e) => write!(f, "IO error: {}", e),
            FramebufferError::InvalidCoordinate { x, y } => {
                write!(f, "Invalid coordinate: ({}, {})", x, y)
            }
            FramebufferError::FontLoadError(msg) => write!(f, "Font load error: {}", msg),
            FramebufferError::FontNotLoaded(font_id) => write!(f, "Font not loaded: {:?}", font_id),
            FramebufferError::DeviceError(msg) => write!(f, "Device error: {}", msg),
            FramebufferError::InvalidDeviceCapabilities(msg) => write!(f, "Invalid device capabilities: {}", msg),
            FramebufferError::BufferSizeMismatch { expected, actual } => {
                write!(f, "Buffer size mismatch: expected {}, got {}", expected, actual)
            }
        }
    }
}

impl std::error::Error for FramebufferError {}

impl From<io::Error> for FramebufferError {
    fn from(error: io::Error) -> Self {
        FramebufferError::IoError(error)
    }
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub width: u16,
    pub height: u16,
    pub bits_per_pixel: u32,
    pub line_length: u32,
    pub buffer_size: usize,
}

impl DeviceInfo {
    fn validate(&self) -> Result<(), FramebufferError> {
        if self.width != DISPLAY_WIDTH {
            return Err(FramebufferError::InvalidDeviceCapabilities(
                format!("Width mismatch: expected {}, got {}", DISPLAY_WIDTH, self.width)
            ));
        }
        
        if self.height != DISPLAY_HEIGHT {
            return Err(FramebufferError::InvalidDeviceCapabilities(
                format!("Height mismatch: expected {}, got {}", DISPLAY_HEIGHT, self.height)
            ));
        }
        
        if self.bits_per_pixel != 32 {
            return Err(FramebufferError::InvalidDeviceCapabilities(
                format!("Bits per pixel mismatch: expected 32, got {}", self.bits_per_pixel)
            ));
        }
        
        let expected_size = (self.line_length as usize) * (self.height as usize);
        if self.buffer_size < expected_size {
            return Err(FramebufferError::BufferSizeMismatch {
                expected: expected_size,
                actual: self.buffer_size,
            });
        }
        
        Ok(())
    }
}

struct PixelBuffer<'a> {
    data: &'a mut [u8],
    width: u16,
    height: u16,
    stride: usize,
}

impl<'a> PixelBuffer<'a> {
    fn new(data: &'a mut [u8], width: u16, height: u16, stride: usize) -> Result<Self, FramebufferError> {
        let required_size = stride * (height as usize);
        if data.len() < required_size {
            return Err(FramebufferError::BufferSizeMismatch {
                expected: required_size,
                actual: data.len(),
            });
        }
        
        Ok(Self {
            data,
            width,
            height,
            stride,
        })
    }
    
    #[inline]
    fn set_pixel(&mut self, x: u16, y: u16, color: Color) -> Result<(), FramebufferError> {
        if x >= self.width || y >= self.height {
            return Err(FramebufferError::InvalidCoordinate { x, y });
        }
        
        let offset = (y as usize) * self.stride + (x as usize) * BYTES_PER_PIXEL;
        
        if offset + BYTES_PER_PIXEL > self.data.len() {
            return Err(FramebufferError::InvalidCoordinate { x, y });
        }
        
        if color.a == 255 {
            let rgba = color.to_bytes();
            self.data[offset..offset + 4].copy_from_slice(&rgba);
        } else if color.a > 0 {
            let bg_r = self.data[offset];
            let bg_g = self.data[offset + 1];
            let bg_b = self.data[offset + 2];
            
            let alpha = color.a as u32;
            let inv_alpha = 255 - alpha;
            
            self.data[offset] = ((color.r as u32 * alpha + bg_r as u32 * inv_alpha) / 255) as u8;
            self.data[offset + 1] = ((color.g as u32 * alpha + bg_g as u32 * inv_alpha) / 255) as u8;
            self.data[offset + 2] = ((color.b as u32 * alpha + bg_b as u32 * inv_alpha) / 255) as u8;
            self.data[offset + 3] = 255;
        }
        
        Ok(())
    }
    
    fn clear(&mut self, color: Color) -> Result<(), FramebufferError> {
        let rgba = color.to_bytes();
        
        for y in 0..self.height {
            let row_start = (y as usize) * self.stride;
            let row_end = row_start + (self.width as usize) * BYTES_PER_PIXEL;
            
            if row_end > self.data.len() {
                return Err(FramebufferError::BufferSizeMismatch {
                    expected: row_end,
                    actual: self.data.len(),
                });
            }
            
            for x in 0..self.width {
                let offset = row_start + (x as usize) * BYTES_PER_PIXEL;
                self.data[offset..offset + 4].copy_from_slice(&rgba);
            }
        }
        
        Ok(())
    }
}

/// Framebuffer interface for direct display access
pub struct Framebuffer {
    mmap: MmapMut,
    device_info: DeviceInfo,
    fonts: HashMap<FontId, Font>,
    glyph_cache: HashMap<(FontId, char, u32), (Vec<u8>, usize, usize)>,
}

impl Framebuffer {
    /// Creates a new framebuffer instance for the given device path
    pub fn new(device_path: &str) -> Result<Self, FramebufferError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_SYNC)
            .open(device_path)?;

        let device_info = Self::get_device_info(&file)?;
        
        device_info.validate()?;

        let mmap = unsafe {
            MmapOptions::new()
                .len(device_info.buffer_size)
                .map_mut(&file)
                .map_err(|e| FramebufferError::DeviceError(format!("Failed to mmap framebuffer: {}", e)))?
        };

        Ok(Self {
            mmap,
            device_info,
            fonts: HashMap::new(),
            glyph_cache: HashMap::new(),
        })
    }
    
    fn get_device_info(file: &File) -> Result<DeviceInfo, FramebufferError> {
        use std::mem;
        
        let fd = file.as_raw_fd();
        
        #[repr(C)]
        struct FbVarScreeninfo {
            xres: u32,
            yres: u32,
            xres_virtual: u32,
            yres_virtual: u32,
            xoffset: u32,
            yoffset: u32,
            bits_per_pixel: u32,
            grayscale: u32,
            padding: [u8; 128],
        }
        
        #[repr(C)]
        struct FbFixScreeninfo {
            id: [u8; 16],
            smem_start: usize,
            smem_len: u32,
            type_: u32,
            type_aux: u32,
            visual: u32,
            xpanstep: u16,
            ypanstep: u16,
            ywrapstep: u16,
            line_length: u32,
            padding: [u8; 32],
        }
        
        const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;
        const FBIOGET_FSCREENINFO: libc::c_ulong = 0x4602;
        
        let mut var_info: FbVarScreeninfo = unsafe { mem::zeroed() };
        let mut fix_info: FbFixScreeninfo = unsafe { mem::zeroed() };
        
        let var_result = unsafe {
            libc::ioctl(fd, FBIOGET_VSCREENINFO, &mut var_info as *mut _)
        };
        
        let fix_result = unsafe {
            libc::ioctl(fd, FBIOGET_FSCREENINFO, &mut fix_info as *mut _)
        };
        
        if var_result == 0 && fix_result == 0 {
            Ok(DeviceInfo {
                width: var_info.xres as u16,
                height: var_info.yres as u16,
                bits_per_pixel: var_info.bits_per_pixel,
                line_length: fix_info.line_length,
                buffer_size: fix_info.smem_len as usize,
            })
        } else {
            Ok(DeviceInfo {
                width: DISPLAY_WIDTH,
                height: DISPLAY_HEIGHT,
                bits_per_pixel: 32,
                line_length: (FRAMEBUFFER_STRIDE as u32) * BYTES_PER_PIXEL as u32,
                buffer_size: FRAMEBUFFER_SIZE,
            })
        }
    }
    
    /// Loads a font from the specified path or uses the default path for the font ID
    pub fn load_font(&mut self, font_id: FontId, font_path: Option<&str>) -> Result<(), FramebufferError> {
        let path = font_path.unwrap_or_else(|| font_id.default_path());
        let font = Self::load_font_from_path(path)?;
        self.fonts.insert(font_id, font);
        Ok(())
    }
    
    fn load_font_from_path(font_path: &str) -> Result<Font, FramebufferError> {
        let font_data = std::fs::read(font_path)
            .map_err(|e| FramebufferError::FontLoadError(format!("Failed to read font file '{}': {}", font_path, e)))?;
        
        let font_settings = FontSettings {
            scale: 40.0,
            ..FontSettings::default()
        };
        
        Font::from_bytes(font_data, font_settings)
            .map_err(|e| FramebufferError::FontLoadError(format!("Failed to parse font '{}': {}", font_path, e)))
    }

    /// Sets a single pixel at the specified coordinates
    #[inline]
    pub fn set_pixel(&mut self, x: u16, y: u16, color: Color) -> Result<(), FramebufferError> {
        let mut buffer = PixelBuffer::new(
            &mut self.mmap[..],
            self.device_info.width,
            self.device_info.height,
            self.device_info.line_length as usize,
        )?;
        
        buffer.set_pixel(x, y, color)
    }

    /// Clears the entire screen with the specified color
    pub fn clear_screen(&mut self, color: Color) -> Result<(), FramebufferError> {
        let mut buffer = PixelBuffer::new(
            &mut self.mmap[..],
            self.device_info.width,
            self.device_info.height,
            self.device_info.line_length as usize,
        )?;
        
        buffer.clear(color)
    }
    
    /// Renders text at the specified position with the given font and color
    pub fn write_text(
        &mut self,
        text: &str,
        position: Point,
        size: f32,
        color: Color,
        font_id: FontId,
    ) -> Result<(), FramebufferError> {
        if !self.fonts.contains_key(&font_id) {
            return Err(FramebufferError::FontNotLoaded(font_id));
        }
        
        let mut cursor_x = position.x as f32;
        let cursor_y = position.y as i32;

        let mut max_ascent = 0;
        for ch in text.chars() {
            if ch == ' ' { continue; }
            let font = self.fonts.get(&font_id).unwrap();
            let (metrics, _) = font.rasterize(ch, size);
            let ascent = metrics.height as i32 + metrics.ymin;
            max_ascent = max_ascent.max(ascent);
        }

        for ch in text.chars() {
            if ch == ' ' {
                cursor_x += size * 0.3;
                continue;
            }

            let size_key = (size * 100.0) as u32;
            let cache_key = (font_id, ch, size_key);
            
            let (metrics, bitmap) = if let Some((cached_bitmap, _width, _height)) = self.glyph_cache.get(&cache_key) {
                let font = self.fonts.get(&font_id).unwrap();
                let (metrics, _) = font.rasterize(ch, size);
                (metrics, cached_bitmap.clone())
            } else {
                let font = self.fonts.get(&font_id).unwrap();
                let (metrics, bitmap) = font.rasterize(ch, size);
                self.glyph_cache.insert(cache_key, (bitmap.clone(), metrics.width, metrics.height));
                (metrics, bitmap)
            };
            
            let glyph_x = (cursor_x + metrics.xmin as f32).round() as i32;
            let char_ascent = metrics.height as i32 + metrics.ymin;
            let glyph_y = cursor_y - max_ascent + (max_ascent - char_ascent);

            for y in 0..metrics.height {
                for x in 0..metrics.width {
                    let pixel_x = glyph_x + x as i32;
                    let pixel_y = glyph_y + y as i32;

                    if pixel_x >= 0 && pixel_x < self.device_info.width as i32 &&
                       pixel_y >= 0 && pixel_y < self.device_info.height as i32 {
                        
                        let bitmap_index = y * metrics.width + x;
                        if bitmap_index < bitmap.len() {
                            let alpha = bitmap[bitmap_index];
                            if alpha > 0 {
                                let blended_color = Color::new(color.r, color.g, color.b, alpha);
                                self.set_pixel(pixel_x as u16, pixel_y as u16, blended_color)?;
                            }
                        }
                    }
                }
            }

            cursor_x += metrics.advance_width;
        }

        Ok(())
    }
    
    /// Flushes any pending operations (currently a no-op)
    #[inline]
    pub fn flush(&self) -> Result<(), FramebufferError> {
        Ok(())
    }
}
