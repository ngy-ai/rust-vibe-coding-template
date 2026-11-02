use app_core::{CoreError, MAX_INPUT_CHARS, execute};

#[test]
fn trims_surrounding_whitespace() {
    let output = execute("  hello \n").expect("非空输入应当成功");
    assert_eq!(output.message, "已处理 5 个字符");
}

#[test]
fn rejects_blank_input() {
    assert_eq!(execute(" \t\n "), Err(CoreError::EmptyInput));
}

#[test]
fn rejects_input_over_limit() {
    let input = "a".repeat(MAX_INPUT_CHARS + 1);
    let expected = CoreError::TooLong {
        len: MAX_INPUT_CHARS + 1,
        max: MAX_INPUT_CHARS,
    };

    assert_eq!(execute(&input), Err(expected));
}

#[test]
fn counts_chars_not_bytes() {
    // 汉字占 3 字节，按字符计数时应恰好卡在上限内。
    let input = "字".repeat(MAX_INPUT_CHARS);
    let output = execute(&input).expect("刚好达到上限应当成功");

    assert_eq!(output.message, format!("已处理 {MAX_INPUT_CHARS} 个字符"));
}
