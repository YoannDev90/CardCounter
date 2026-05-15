use gpui::*;
use rxing::{common::HybridBinarizer, BinaryBitmap, MultiUseMultiFormatReader, Reader, DecodingHintDictionary, DecodeHintValue, DecodeHintType, BarcodeFormat};
use nokhwa::pixel_format::RgbFormat;
use std::collections::HashSet;
use nokhwa::utils::{CameraIndex, ControlValueSetter, KnownCameraControl, RequestedFormat, RequestedFormatType};
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
    ) -> Task<()> {
        let handle_data = Arc::new(handle_data);
        println!("[DEBUG] Appels Scanner::start - Tentative spawn");
        let task = cx.spawn(|_this: WeakEntity<T>, cx_ref: &mut AsyncApp| {
            println!("[DEBUG] Entrée dans la closure de spawn (sur thread worker)");
            let cx = cx_ref.clone();
            let mut throttle = ScanThrottle::new();
            let this = weak_handle;
            
            async move {
                println!("[SCANNER] Tâche asynchrone lancée via async move");
                
                let mut loop_count = 0u64;
                println!("[SCANNER] Enumération des caméras...");
                match nokhwa::query(nokhwa::utils::ApiBackend::Auto) {
                    Ok(devices) => {
                        println!("[SCANNER] {} caméras trouvées", devices.len());
                        for dev in devices {
                            println!("[SCANNER] Caméra: {}", dev.human_name());
                        }
                    }
                    Err(e) => println!("[SCANNER] Erreur énumération: {:?}", e),
                }

                let index = CameraIndex::Index(0);
                // On repasse en mode automatique le plus fluide mais on va tenter de stabiliser
                let requested = RequestedFormat::new::<RgbFormat>(
                    RequestedFormatType::AbsoluteHighestFrameRate,
                );
                
                println!("[SCANNER] Tentative de création de la caméra (mode auto)...");
                let mut camera = match Camera::new(index, requested) {
                    Ok(c) => {
                        println!("[SCANNER] Instance caméra créée.");
                        c
                    },
                    Err(e) => {
                        eprintln!("[SCANNER] Erreur critique Camera::new: {:?}", e);
                        return;
                    }
                };

                println!("[SCANNER] Ouverture du stream...");
                if let Err(e) = camera.open_stream() {
                    eprintln!("[SCANNER] Erreur critique open_stream: {:?}", e);
                    return;
                }
                println!("[SCANNER] Stream ouvert ! La LED de la caméra devrait s'allumer.");

                // Tentative d'activation de l'Autofocus
                match camera.camera_controls() {
                    Ok(controls) => {
                        for control in controls {
                            if control.control() == nokhwa::utils::KnownCameraControl::Focus {
                                println!("[SCANNER] Contrôle de focus trouvé. Tentative d'activation de l'autofocus...");
                                let mut auto_focus = control.clone();
                                // On tente d'activer le flag manuel si déjà présent ou auto
                                if let Err(e) = camera.set_camera_control(KnownCameraControl::Focus, ControlValueSetter::None) {
                                    println!("[SCANNER] Échec activation autofocus: {:?}", e);
                                } else {
                                    println!("[SCANNER] Autofocus activé (si supporté par le driver).");
                                }
                            }
                        }
                    }
                    Err(e) => println!("[SCANNER] Impossible de lister les contrôles: {:?}", e),
                }

                let mut frame_count = 0u64;
                let mut last_fps_check = Instant::now();
                let mut fps = 0.0;

                loop {
                    loop_count += 1;
                    
                    // Calcul des FPS toutes les secondes
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_fps_check);
                    if elapsed >= Duration::from_secs(1) {
                        fps = frame_count as f64 / elapsed.as_secs_f64();
                        println!("[SCANNER] FPS: {:.1} | Itération #{}", fps, loop_count);
                        frame_count = 0;
                        last_fps_check = now;
                    }

                    let mut is_interactive = false;
                    if let Some(view) = this.upgrade() {
                        let _ = cx.update(|cx| {
                            is_interactive = mode_check(view.read(cx)) == AppMode::Interactive;
                        });
                    }

                    if is_interactive {
                        // Cruciaux : On cède la main à l'exécuteur GPUI pour éviter de bloquer le thread worker
                        // et on laisse le temps au thread principal de traiter les cx.update
                        smol::Timer::after(Duration::from_millis(1)).await;

                        match camera.frame() {
                            Ok(frame) => {
                                frame_count += 1;
                                if frame_count % 100 == 0 {
                                    println!("[SCANNER] Capture frame #{}", frame_count);
                                }

                                if let Ok(decoded_frame) = frame.decode_image::<RgbFormat>() {
                                    let width = decoded_frame.width();
                                    let height = decoded_frame.height();
                                    
                                    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);
                                    let mut gray_pixels = Vec::with_capacity((width * height) as usize);

                                    for p in decoded_frame.pixels() {
                                        // Conversion BGR -> RGBA pour GPUI (nokhwa renvoie souvent du BGR par défaut sur Linux/V4L2)
                                        rgba_data.push(p.0[2]); // R (index 2)
                                        rgba_data.push(p.0[1]); // G (index 1)
                                        rgba_data.push(p.0[0]); // B (index 0)
                                        rgba_data.push(255);
                                        // Grayscale pour masuri
                                        gray_pixels.push(((p.0[0] as u32 + p.0[1] as u32 + p.0[2] as u32) / 3) as u8);
                                    }

                                    if let Some(buf) = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, rgba_data) {
                                        let frame_obj = Frame::new(buf);
                                        let render_img = Arc::new(RenderImage::new(vec![frame_obj]));
                                        let src = ImageSource::Render(render_img);

                                        if let Some(view) = this.upgrade() {
                                            let _ = cx.update(|cx| {
                                                let _ = view.update(cx, |v, cx| {
                                                    update_frame(v, src, cx);
                                                });
                                            });
                                        }

                                        // Décodage parallélisé pour ne pas laguer la preview
                                        if frame_count % 3 == 0 {
                                            let gray_for_decode = gray_pixels.clone();
                                            let last_code = throttle.last_code.clone();
                                            let last_scan = throttle.last_scan;
                                            
                                            // Clones pour le spawn
                                            let view_clone = this.clone();
                                            let cx_clone = cx.clone();
                                            let handle_data_clone = handle_data.clone();

                                            cx.foreground_executor()
                                                .spawn(async move {
                                                    let mut hints = DecodingHintDictionary::new();
                                                    let mut formats = HashSet::new();
                                                    formats.insert(BarcodeFormat::ITF);
                                                    formats.insert(BarcodeFormat::CODE_128);
                                                    formats.insert(BarcodeFormat::EAN_13);

                                                    hints.insert(DecodeHintType::POSSIBLE_FORMATS, DecodeHintValue::PossibleFormats(formats));
                                                    hints.insert(DecodeHintType::TRY_HARDER, DecodeHintValue::TryHarder(true));

                                                    let mut reader = MultiUseMultiFormatReader::default();
                                                    let luma_source = rxing::Luma8LuminanceSource::new(gray_for_decode, width, height);
                                                    let binarizer = HybridBinarizer::new(luma_source);
                                                    let mut bitmap = BinaryBitmap::new(binarizer);

                                                    if let Ok(result) = reader.decode_with_hints(&mut bitmap, &hints) {
                                                        let code = result.getText().trim().to_string();
                                                        
                                                        if !code.is_empty() {
                                                            let now = Instant::now();
                                                            if code != last_code || now.duration_since(last_scan) > Duration::from_secs(2) {
                                                                println!("[SCANNER] Détecté via RXing: {} ({:?})", code, result.getBarcodeFormat());
                                                                let _ = cx_clone.update(|cx| {
                                                                    if let Some(view) = view_clone.upgrade() {
                                                                        let _ = view.update(cx, |v, cx| {
                                                                            handle_data_clone(v, &code, cx);
                                                                        });
                                                                    }
                                                                });
                                                            }
                                                        }
                                                    }
                                                })
                                                .detach();
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("[SCANNER] Erreur frame: {:?}", e);
                                smol::Timer::after(Duration::from_millis(100)).await;
                            }
                        }
                    } else {
                        smol::Timer::after(Duration::from_millis(100)).await;
                    }
                }
            }
        });
        task
    }
}
