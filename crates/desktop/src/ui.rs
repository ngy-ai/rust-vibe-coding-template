use app_core::{CoreError, Output, execute};
use gpui_kit::*;

/// 把共享核心的结果映射成界面要展示的文案与是否错误态。
///
/// 抽成纯函数是为了可测：`run_core` 需要 `Context<Self>`，没有 gpui 运行时无法构造，
/// 而「成功显示 message、失败显示错误文案」正是各端唯一的界面逻辑，值得被测试锁住。
fn to_view(result: Result<Output, CoreError>) -> (String, bool) {
    match result {
        Ok(output) => (output.message, false),
        Err(error) => (error.to_string(), true),
    }
}

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
        let (result, is_error) = to_view(execute(&self.sample));
        self.result = result;
        self.is_error = is_error;
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

#[cfg(test)]
mod tests {
    // 只引入被测函数与核心入口：`use super::*` 会把 `gpui_kit::*` 里的宏再导进来，
    // 与 `#[test]` 的属性展开相撞（recursion limit reached）。
    use app_core::execute;

    use super::to_view;

    #[test]
    fn success_shows_shared_message() {
        let (text, is_error) = to_view(execute("hello"));

        assert_eq!(text, "已处理 5 个字符");
        assert!(!is_error);
    }

    #[test]
    fn blank_input_shows_core_error_text() {
        let (text, is_error) = to_view(execute("   "));

        assert!(is_error, "空白输入必须进入错误态");
        assert!(text.contains("不能为空"), "应展示核心文案，实际: {text}");
        assert!(text.contains("hello"), "错误文案要给出可执行的示例");
    }

    #[test]
    fn too_long_input_reports_actual_length() {
        let (text, is_error) = to_view(execute(&"字".repeat(4097)));

        assert!(is_error);
        assert!(text.contains("4097"), "应包含实际字符数，实际: {text}");
        assert!(text.contains("4096"), "应包含上限，实际: {text}");
    }
}
