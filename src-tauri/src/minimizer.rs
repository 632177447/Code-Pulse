pub fn minimize_code(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;
    
    // 状态追踪：字符串和注释
    let mut in_string: Option<char> = None;
    let mut in_comment: Option<char> = None; // '/' 代表 //, '*' 代表 /* */

    while i < chars.len() {
        let c = chars[i];

        // --- 状态更新 ---
        if in_comment.is_none() && in_string.is_none() {
            if c == '"' || c == '\'' || c == '`' {
                in_string = Some(c);
            } else if c == '/' && i + 1 < chars.len() {
                if chars[i + 1] == '/' {
                    in_comment = Some('/');
                } else if chars[i + 1] == '*' {
                    in_comment = Some('*');
                }
            }
        } else if let Some(q) = in_string {
            // 处理字符串结束（考虑转义）
            if c == q && (i == 0 || chars[i - 1] != '\\') {
                in_string = None;
            }
        } else if let Some(com) = in_comment {
            // 处理注释结束
            if com == '/' && c == '\n' {
                in_comment = None;
            } else if com == '*' && c == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                in_comment = None;
                result.push('*');
                result.push('/');
                i += 2;
                continue;
            }
        }

        // --- 压缩逻辑 ---
        if in_string.is_none() && in_comment.is_none() && c == '{' {
            if should_compress(&chars, i) {
                result.push('{');
                
                // 查找匹配的 '}'，同时跳过其内部的字符串和注释
                let mut stack = 1;
                let mut j = i + 1;
                let mut inner_string = None;
                let mut inner_comment = None;
                let mut found = false;

                while j < chars.len() {
                    let ic = chars[j];
                    if inner_comment.is_none() && inner_string.is_none() {
                        if ic == '"' || ic == '\'' || ic == '`' {
                            inner_string = Some(ic);
                        } else if ic == '/' && j + 1 < chars.len() {
                            if chars[j + 1] == '/' { inner_comment = Some('/'); }
                            else if chars[j + 1] == '*' { inner_comment = Some('*'); }
                        } else if ic == '{' {
                            stack += 1;
                        } else if ic == '}' {
                            stack -= 1;
                            if stack == 0 { found = true; break; }
                        }
                    } else if let Some(q) = inner_string {
                        if ic == q && (j == 0 || chars[j - 1] != '\\') { inner_string = None; }
                    } else if let Some(com) = inner_comment {
                        if com == '/' && ic == '\n' { inner_comment = None; }
                        else if com == '*' && ic == '*' && j + 1 < chars.len() && chars[j + 1] == '/' {
                            inner_comment = None;
                            j += 1;
                        }
                    }
                    j += 1;
                }

                if found {
                    result.push_str(" /* ... */ ");
                    result.push('}');
                    i = j + 1;
                    continue;
                }
            }
        }

        result.push(c);
        i += 1;
    }

    result
}

/// 启发式判断是否应该压缩当前大括号块
/// 目标是仅压缩函数体和控制流块，排除 import/export/声明/对象字面量
fn should_compress(chars: &[char], pos: usize) -> bool {
    let mut i = pos as i32 - 1;
    let mut current_word = Vec::new();

    while i >= 0 {
        let c = chars[i as usize];

        if c.is_whitespace() {
            if !current_word.is_empty() {
                current_word.reverse();
                let w: String = current_word.iter().collect();
                if is_allow_keyword(&w) { return true; }
                if is_deny_keyword(&w) { return false; }
                current_word.clear();
            }
        } else if c == ')' {
            // 函数签名结束，极大概率后面是函数体
            return true;
        } else if c == '>' && i > 0 && chars[i as usize - 1] == '=' {
            // 箭头函数 =>
            return true;
        } else if c == ';' || c == '{' || c == '}' || c == '(' || c == '=' || c == ',' {
            // 碰到这些符号（除冒号、中括号外），说明当前块极大概率是对象属性、数组元素或非法起始
            // 允许冒号、中括号、尖括号以支持复杂的 TypeScript 类型声明
            return false;
        } else {
            current_word.push(c);
        }
        i -= 1;
    }

    if !current_word.is_empty() {
        current_word.reverse();
        let w: String = current_word.iter().collect();
        if is_allow_keyword(&w) { return true; }
        if is_deny_keyword(&w) { return false; }
    }

    false
}

fn is_allow_keyword(w: &str) -> bool {
    matches!(w, "else" | "try" | "do" | "finally" | "static" | "unsafe" | "fn" | "func" | "function" | "def" | "get" | "set" | "async")
}

fn is_deny_keyword(w: &str) -> bool {
    matches!(w, "import" | "export" | "const" | "let" | "var" | "interface" | "type" | "enum" | "struct" | "class" | "impl" | "trait" | "return" | "yield" | "throw")
}
