use gpui::*;
use masuri::{decode_parallel, SymbolType};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;
use std::sync::Arc;
use crate::types::{AppMode, ScanThrottle};
use image::{ImageBuffer, Rgba, Frame};
use std::time::{Duration, Instant};

pub struct Scanner;

impl Scanner {
    pub fn start<T: 'static + Render>(
        weak_handle: WeakEntity<T>,
        cx: &mut Context<T>,
        mode_check: impl Fn(&T) -> AppMode + Send + Sync + 'static,
        handle_data: impl Fn(&mut T, &str, &mut Context<T>) + Send + Sync + 'static,
        update_frame: impl Fn(&mut T, ImageSource, &mut Context<T>) + Send + Sync + 'static,
    ) {
        cx.spawn(|_this: WeakEntity<T>, cx_ref: &mut AsyncApp| {
            let mut cx = cx_ref.clone();
            let this = weak_handle;
            let mut throttle = ScanThrottle::new();
            
            async move {
                let index = CameraIndex::Index(0);
                let requested = RequestedFormat::new::<RgbFormat>(
                    RequestedFormatType::AbsoluteHighestFrameRate,
                );
                let mut camera = match Camera::new(index, requested) {
                    Ok(c) => c,
                    Err(_) => return,
                };

                if camera.open_stream().is_err() {
                    return;
                }

                loop {
                    let mut is_interactive = false;
                    let _ = this.upgrade().map(|view: Entity<T>| {
                        if let Ok(m) = view.read_with(&cx, |v, _| mode_check(v)) {
                            is_interactive = m == AppMode::Interactive;
                        }
                    });

                    if is_interactive {
                        if let Ok(frame) = camera.frame() {
                            if let Ok(decoded_frame) = frame.decode_image::<RgbFormat>() {
                                let width = decoded_frame.width();
                                let height = decoded_frame.height();
                                
                                // Direct conversion to RGBA for GPUI
                                let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);
                                for p in decoded_frame.pixels() {
                                    rgba_data.push(p.0[0]);
                                    rgba_data.push(p.0[1]);
                                    rgba_data.push(p.0[2]);
                                    rgba_data.push(255);
                                }

                                // Decode Barcode (Gray)
                                let gray_pixels: Vec<u8> = decoded_frame
                                    .pixels()
                                    .map(|p| (p.0[0] as u32 + p.0[1] as u32 + p.0[2] as u32) / 3)
                                    .map(|v| v as u8)
                                    .collect();

                                let results = decode_parallel(&gray_pixels, width, height);

                                let buf: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, rgba_data).unwrap();
                                let frame = Frame::new(buf);
                                let render_img = Arc::new(RenderImage::new(vec![frame]));
                                let src = ImageSource::Render(render_img);

                                // Update UI ASAP for high framerate
                                let _ = this.upgrade().map(|view: Entity<T>| {
                                    let _ = cx.update(|cx| {
                                        let _ = view.update(cx, |v: &mut T, cx| {
                                            update_frame(v, src, cx);
                                        });
                                        Ok::<(), ()>(())
                                    });
                                });

                                for result in results {
                                    if result.sym_type == SymbolType::Code128 {
                                        let code = result.data.trim().to_string();
                                        
                                        // "Delicate" Antiflicker:
                                        // 1. Same code -> wait at least 2s
                                        // 2. Different code -> immediate scan allowed
                                        let now = Instant::now();
                                        let can_scan = if code == throttle.last_code {
                                            now.duration_since(throttle.last_scan) > Duration::from_secs(2)
                                        } else {
                                            true
                                        };

                                        if can_scan {
                                            throttle.last_code = code.clone();
                                            throttle.last_scan = now;
                                            
                                            let _ = this.upgrade().map(|view: Entity<T>| {
                                                let _ = cx.update(|cx| {
                                                    let _ = view.update(cx, |v: &mut T, cx| {
                                                        handle_data(v, &code, cx);
                                                    });
                                                    Ok::<(), ()>(())
                                                });
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        smol::Timer::after(Duration::from_millis(100)).await;
                    }
                    // Yield quickly for higher FPS
                    smol::future::yield_now().await;
                }
            }
        })
        .detach();
    }
}
