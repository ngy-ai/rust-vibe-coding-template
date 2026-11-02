use app_core::execute;
use gpui_kit::*;

/// 主窗口根视图。
///
/// 视图只持有界面状态，业务规则与错误文案全部来自 [`app_core`]，桌面端不重复实现校验。
pub(crate) struct Root {
    /// 本次调用的输入；每次点击后追加序号，便于肉眼看出一致性。
    sample: String,
    /// 上一次调用的结果或错误文案。
    result: String,
    is_error: bool,
}

impl Root {
    pub(crate) fn new(_: &mut Context<Self>) -> Self {
        Self {
            sample: String::from("hello"),
            result: String::from("点击按钮调用共享核心"),
            is_error: false,
        }
    }

    fn run_core(&mut self, cx: &mut Context<Self>) {
        self.sample.push('.');
        match execute(&self.sample) {
            Ok(output) => {
                self.result = output.message;
                self.is_error = false;
            }
            Err(error) => {
                self.result = error.to_string();
                self.is_error = true;
            }
        }
        cx.notify();
    }
}

impl Render for Root {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let result_color = if self.is_error {
            rgb(0xf8_71_71)
        } else {
            rgb(0x4a_de_80)
        };

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_8()
            .size_full()
            .bg(rgb(0x18_18_1b))
            .text_color(rgb(0xe4_e4_e7))
            .child(div().text_2xl().child("Rust AI Template"))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xa1_a1_aa))
                    .child("桌面端：gpui-kit 负责界面，app-core 负责业务"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_4()
                    .rounded_lg()
                    .bg(rgb(0x27_27_2a))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xa1_a1_aa))
                            .child("共享核心输入"),
                    )
                    .child(div().text_lg().child(self.sample.clone())),
            )
            .child(
                div()
                    .id("run-core")
                    .px_4()
                    .py_2()
                    .rounded_md()
                    .bg(rgb(0x25_63_eb))
                    .text_color(rgb(0xfa_fa_fa))
                    .cursor_pointer()
                    .child("运行共享核心")
                    .on_click(cx.listener(|this, _, _, cx| this.run_core(cx))),
            )
            .child(div().text_color(result_color).child(self.result.clone()))
    }
}
