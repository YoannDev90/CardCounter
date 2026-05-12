#![recursion_limit = "1024"]
use gpui::prelude::*;
use gpui::*;
use std::sync::Arc;

mod db;
use db::Db;

struct CardCounter {
    db: Arc<Db>,
    current_input: String,
    last_result: Option<bool>,
    last_code: String,
    focus_handle: FocusHandle,
    show_reset_confirm: bool,
}

impl CardCounter {
    fn new(db: Arc<Db>, cx: &mut Context<Self>) -> Self {
        Self {
            db,
            current_input: String::new(),
            last_result: None,
            last_code: String::from("------"),
            focus_handle: cx.focus_handle(),
            show_reset_confirm: false,
        }
    }

    fn beep_ok(&self) {
        print!("\x07"); // System bell
    }

    fn beep_error(&self) {
        // Double beep for error/dupe
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

        let focus_handle = self.focus_handle.clone();

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .bg(bg_color)
            .relative()
            .on_key_down(cx.listener(
                |view: &mut Self,
                 event: &KeyDownEvent,
                 _window: &mut Window,
                 cx: &mut Context<Self>| {
                    view.handle_input(&event.keystroke.key, cx);
                },
            ))
            .track_focus(&focus_handle)
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .child(status)
                    .text_color(color),
            )
            .child(
                div()
                    .mt_12()
                    .text_2xl()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0xffffff))
                    .bg(rgb(0x000000))
                    .px_8()
                    .py_4()
                    .rounded_xl()
                    .child(self.last_code.clone()),
            )
            .child(
                div()
                    .mt_16()
                    .flex()
                    .gap_4()
                    .child(div().text_xl().text_color(rgb(0xcccccc)).child("Saisie:"))
                    .child(
                        div()
                            .text_2xl()
                            .text_color(rgb(0xffffff))
                            .font_weight(FontWeight::BOLD)
                            .child(self.current_input.clone()),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .bottom_8()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x888888))
                            .child("ATTENTE SCAN (6 CHIFFRES) | VERIFIEZ NUMLOCK"),
                    )
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .bg(rgb(0x333333))
                            .rounded_md()
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(Self::toggle_reset_confirm),
                            )
                            .child("RESET DATA"),
                    ),
            )
            .when(self.show_reset_confirm, |this| {
                this.child(
                    div()
                        .absolute()
                        .size_full()
                        .bg(rgba(0x000000aa))
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .bg(rgb(0x222222))
                                .p_8()
                                .rounded_xl()
                                .border_1()
                                .border_color(rgb(0x444444))
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
                                                .cursor_pointer()
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(Self::confirm_reset),
                                                )
                                                .child("OUI, TOUT EFFACER"),
                                        )
                                        .child(
                                            div()
                                                .px_6()
                                                .py_2()
                                                .bg(rgb(0x444444))
                                                .rounded_md()
                                                .cursor_pointer()
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(Self::toggle_reset_confirm),
                                                )
                                                .child("ANNULER"),
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
