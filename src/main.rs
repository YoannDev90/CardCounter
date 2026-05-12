use gpui::prelude::*;
use gpui::*;
use std::sync::Arc;

mod db;
mod types;
mod scanner;

use db::Db;
use types::AppMode;
use scanner::Scanner;

struct CardCounter {
    db: Arc<Db>,
    current_input: String,
    last_result: Option<bool>,
    last_code: String,
    focus_handle: FocusHandle,
    show_reset_confirm: bool,
    mode: AppMode,
    camera_frame: Option<ImageSource>,
}

impl CardCounter {
    fn new(db: Arc<Db>, cx: &mut Context<Self>) -> Self {
        let view = Self {
            db,
            current_input: String::new(),
            last_result: None,
            last_code: String::from("------"),
            focus_handle: cx.focus_handle(),
            show_reset_confirm: false,
            mode: AppMode::Interactive,
            camera_frame: None,
        };

        Scanner::start(
            cx.entity().downgrade(),
            cx,
            |v| v.mode,
            |v, data, cx| v.handle_input(data, cx),
            |v, frame, cx| {
                v.camera_frame = Some(frame);
                cx.notify();
            },
        );
        view
    }

    fn set_mode(&mut self, mode: AppMode, cx: &mut Context<Self>) {
        self.mode = mode;
        if mode == AppMode::Manual {
            self.camera_frame = None;
        }
        cx.notify();
    }

    fn beep_ok(&self) {
        print!("\x07");
    }

    fn beep_error(&self) {
        print!("\x07");
        std::thread::sleep(std::time::Duration::from_millis(100));
        print!("\x07");
    }

    fn handle_input(&mut self, text: &str, cx: &mut Context<Self>) {
        if self.show_reset_confirm {
            return;
        }

        if text.chars().all(|c| c.is_ascii_digit()) {
            self.current_input.push_str(text);
        }

        while self.current_input.len() >= 6 {
            let code = self.current_input[..6].to_string();
            self.current_input = self.current_input[6..].to_string();
            self.last_code = code.clone();

            if let Ok(exists) = self.db.check_code(&code) {
                self.last_result = Some(exists);
                if exists {
                    self.beep_error();
                } else {
                    let _ = self.db.add_code(&code);
                    self.beep_ok();
                }
            }
            cx.notify();
        }
    }

    fn toggle_reset_confirm(&mut self, _: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.show_reset_confirm = !self.show_reset_confirm;
        cx.notify();
    }

    fn confirm_reset(&mut self, _: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let _ = self.db.reset();
        self.show_reset_confirm = false;
        self.last_result = None;
        self.last_code = String::from("RESET!");
        self.beep_ok();
        self.beep_ok();
        cx.notify();
    }
}

impl Focusable for CardCounter {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for CardCounter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (status, color, bg_color) = match self.last_result {
            Some(true) => ("DÉJÀ CONNU", rgb(0xffffff), rgb(0xbb2222)),
            Some(false) => ("NOUVEAU - OK", rgb(0x000000), rgb(0x22bb22)),
            None => ("PRÊT", rgb(0x888888), rgb(0x1a1a1a)),
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(bg_color)
            .relative()
            .on_key_down(cx.listener(|view, event: &KeyDownEvent, _window, cx| {
                view.handle_input(&event.keystroke.key, cx);
            }))
            .track_focus(&self.focus_handle)
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .w_full()
                    .p_4()
                    .bg(rgb(0x000000))
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .rounded_lg()
                                    .bg(if self.mode == AppMode::Manual { rgb(0x444444) } else { rgb(0x1e1e1e) })
                                    .text_color(rgb(0xffffff))
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                        this.set_mode(AppMode::Manual, cx);
                                    }))
                                    .child("⌨️ CLAVIER"),
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .rounded_lg()
                                    .bg(if self.mode == AppMode::Interactive { rgb(0x444444) } else { rgb(0x1e1e1e) })
                                    .text_color(rgb(0xffffff))
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                        this.set_mode(AppMode::Interactive, cx);
                                    }))
                                    .child("📷 WEBCAM"),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_grow()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .w_full()
                    .child(div().text_3xl().font_weight(FontWeight::BOLD).child(status).text_color(color))
                    .child(
                        div()
                            .mt_8()
                            .text_3xl()
                            .text_color(rgb(0xffffff))
                            .bg(rgb(0x000000))
                            .px_10()
                            .py_6()
                            .rounded_xl()
                            .child(self.last_code.clone()),
                    )
                    .when(self.mode == AppMode::Interactive, |parent| {
                        parent.child(
                            div()
                                .mt_8()
                                .size_64()
                                .bg(rgb(0x000000))
                                .rounded_lg()
                                .overflow_hidden()
                                .child(match &self.camera_frame {
                                    Some(src) => div().size_full().child(img(src.clone()).size_full()),
                                    None => div()
                                        .size_full()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_color(rgb(0x888888))
                                        .child("Caméra..."),
                                }),
                        )
                    })
                    .when(self.mode == AppMode::Manual, |parent| {
                        parent.child(
                            div()
                                .mt_12()
                                .flex()
                                .gap_4()
                                .child(div().text_2xl().text_color(rgb(0xcccccc)).child("Saisie:"))
                                .child(div().text_2xl().text_color(rgb(0xffffff)).child(self.current_input.clone())),
                        )
                    }),
            )
            .child(
                div()
                    .absolute()
                    .bottom_4()
                    .right_4()
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .bg(rgb(0x1a1a1a))
                            .text_color(rgb(0xff4444))
                            .rounded_lg()
                            .on_mouse_down(MouseButton::Left, cx.listener(Self::toggle_reset_confirm))
                            .child("RESET DATA"),
                    ),
            )
            .when(self.show_reset_confirm, |this| {
                this.child(
                    div()
                        .absolute()
                        .size_full()
                        .bg(rgba(0x000000))
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .bg(rgb(0x222222))
                                .p_8()
                                .rounded_xl()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap_6()
                                .child(div().text_xl().child("EFFACER TOUTE LA DB ?"))
                                .child(
                                    div()
                                        .flex()
                                        .gap_4()
                                        .child(
                                            div()
                                                .px_6()
                                                .py_2()
                                                .bg(rgb(0xbb2222))
                                                .rounded_md()
                                                .on_mouse_down(MouseButton::Left, cx.listener(Self::confirm_reset))
                                                .child("OUI"),
                                        )
                                        .child(
                                            div()
                                                .px_6()
                                                .py_2()
                                                .bg(rgb(0x444444))
                                                .rounded_md()
                                                .on_mouse_down(MouseButton::Left, cx.listener(Self::toggle_reset_confirm))
                                                .child("NON"),
                                        ),
                                ),
                        ),
                )
            })
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let db = Arc::new(Db::init().expect("Erreur DB"));
        cx.open_window(WindowOptions::default(), move |window, cx| {
            let view = cx.new(|cx| CardCounter::new(db, cx));
            view.focus_handle(cx).focus(window);
            view
        })
        .expect("Erreur ouverture fenêtre");
    });
}
