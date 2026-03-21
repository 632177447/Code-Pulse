pub fn minimize_code(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let c = chars[i];

        // 识别可能的函数定义起始（简化的逻辑：匹配 '{' 且当前不在注释/字符串中）
        // 为了保持最低噪音，我们只在最外层或接近外层的地方触发压缩
        if c == '{' && !is_in_string_or_comment(&chars, i) {
            result.push('{');
            
            // 找到匹配的 '}'
            let mut stack = 1;
            let mut j = i + 1;
            let mut skip_start = j;
            let mut found_match = false;

            while j < chars.len() {
                if !is_in_string_or_comment(&chars, j) {
                    if chars[j] == '{' {
                        stack += 1;
                    } else if chars[j] == '}' {
                        stack -= 1;
                        if stack == 0 {
                            found_match = true;
                            break;
                        }
                    }
                }
                j += 1;
            }

            if found_match && j > skip_start {
                result.push_str(" /* ... */ ");
                i = j; // 跳到 '}'
                result.push('}');
                i += 1;
                continue;
            }
        }

        result.push(c);
        i += 1;
    }

    result
}

// 辅助函数：判断当前字符是否在字符串或注释中
fn is_in_string_or_comment(chars: &[char], pos: usize) -> bool {
    let mut in_string = None;
    let mut in_comment = None;
    let mut i = 0;

    while i < pos {
        let c = chars[i];
        
        if in_comment.is_none() && in_string.is_none() {
            if c == '"' || c == '\'' || c == '`' {
                in_string = Some(c);
            } else if c == '/' && i + 1 < chars.len() {
                if chars[i+1] == '/' {
                    in_comment = Some('/');
                } else if chars[i+1] == '*' {
                    in_comment = Some('*');
                }
            }
        } else if let Some(q) = in_string {
            if c == q && (i == 0 || chars[i-1] != '\\') {
                in_string = None;
            }
        } else if let Some(com) = in_comment {
            if com == '/' && c == '\n' {
                in_comment = None;
            } else if com == '*' && c == '*' && i + 1 < chars.len() && chars[i+1] == '/' {
                in_comment = None;
                i += 1;
            }
        }
        i += 1;
    }
    
    in_string.is_some() || in_comment.is_some()
}
