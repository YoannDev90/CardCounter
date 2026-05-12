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
            let cx = cx_ref.clone();
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

                // Buffer de Gray pour le décodage barcode pour éviter les réallocations
                let mut gray_pixels = Vec::new();
                let mut rgba_data = Vec::new();

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
                                let total_pixels = (width * height) as usize;
                                
                                // Réutilisation des buffers
                                rgba_data.clear();
                                rgba_data.reserve(total_pixels * 4);
                                gray_pixels.clear();
                                gray_pixels.reserve(total_pixels);

                                // Une seule boucle pour RGBA et Gray
                                for p in decoded_frame.pixels() {
                                    let r = p.0[0];
                                    let g = p.0[1];
                                    let b = p.0[2];
                                    
                                    rgba_data.push(r);
                                    rgba_data.push(g);
                                    rgba_data.push(b);
                                    rgba_data.push(255);
                                    
                                    // Luminance (Luma Y' pour rapidité)
                                    gray_pixels.push(((r as u32 + g as u32 + b as u32) / 3) as u8);
                                }

                                // Création de la frame GPUI
                                if let Some(buf) = ImageBuffer::from_raw(width, height, rgba_data.clone()) {
                                    let frame = Frame::new(buf);
                                    let render_img = Arc::new(RenderImage::new(vec![frame]));
                                    let src = ImageSource::Render(render_img);

                                    // Mise à jour UI Immédiate
                                    let _ = this.upgrade().map(|view| {
                                        let _ = cx.update(|cx| {
                                            let _ = view.update(cx, |v, cx| {
                                                update_frame(v, src, cx);
                                            });
                                            Ok::<(), ()>(())
                                        });
                                    });
                                }

                                // Décodage en parallèle (Asynchrone par rapport à l'affichage)
                                let results = decode_parallel(&gray_pixels, width, height);

                                for result in results {
                                    if result.sym_type == SymbolType::Code128 {
                                        let code = result.data.trim().to_string();
                                        let now = Instant::now();
                                        
                                        let can_scan = if code == throttle.last_code {
                                            now.duration_since(throttle.last_scan) > Duration::from_secs(2)
                                        } else {
                                            true
                                        };

                                        if can_scan {
                                            throttle.last_code = code.clone();
                                            throttle.last_scan = now;
                                            
                                            let _ = this.upgrade().map(|view| {
                                                let _ = cx.update(|cx| {
                                                    let _ = view.update(cx, |v, cx| {
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
                        // Très courte attente si en mode manuel pour libérer le CPU
                        smol::Timer::after(Duration::from_millis(100)).await;
                    }
                }
            }
        });
    }
}
