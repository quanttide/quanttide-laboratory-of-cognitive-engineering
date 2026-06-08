use regex::Regex;
use std::collections::HashSet;

const STOPWORDS: &[&str] = &[
    "的", "了", "是", "在", "有", "我", "这", "也", "和", "就", "都", "而", "及", "与",
    "着", "或", "一个", "没有", "我们", "他们", "因为", "所以", "但是", "可以", "这个", "那个",
];

fn is_cjk(c: char) -> bool {
    matches!(c as u32,
        0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0x2E80..=0x2EFF | 0x3000..=0x303F |
        0x31C0..=0x31EF | 0x3200..=0x32FF | 0x3300..=0x33FF | 0xF900..=0xFAFF |
        0xFE30..=0xFE4F
    )
}

fn is_stopword_char(c: char) -> bool {
    matches!(c, '的' | '了' | '是' | '在' | '有' | '我' | '这' | '也' | '和' | '就' |
                  '都' | '而' | '及' | '与' | '着' | '或' | '一' | '个' | '没' | '们' |
                  '因' | '为' | '所' | '以' | '但' | '可')
}

fn extract_bigrams(segment: &str) -> Vec<String> {
    let chars: Vec<char> = segment.chars().collect();
    if chars.len() < 2 {
        return Vec::new();
    }
    let mut tokens = Vec::new();
    for i in 0..chars.len().saturating_sub(1) {
        if is_cjk(chars[i]) && is_cjk(chars[i + 1])
            && !is_stopword_char(chars[i])
            && !is_stopword_char(chars[i + 1])
        {
            let bigram: String = chars[i..=i + 1].iter().collect();
            tokens.push(bigram);
        }
    }
    tokens
}

pub fn tokenize(text: &str) -> Vec<String> {
    let re = Regex::new(r"[[:punct:]\s]+").unwrap();
    let stopword_set: HashSet<&str> = STOPWORDS.iter().copied().collect();

    let segments: Vec<&str> = re.split(text).filter(|s| !s.is_empty()).collect();
    let mut tokens: Vec<String> = Vec::new();

    for segment in segments {
        let has_cjk = segment.chars().any(|c| is_cjk(c));

        if has_cjk {
            tokens.extend(extract_bigrams(segment));
        } else if segment.len() >= 2 && !stopword_set.contains(segment) {
            tokens.push(segment.to_string());
        }
    }

    tokens.sort();
    tokens.dedup();
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_cjk_bigrams() {
        let tokens = tokenize("研发方法论");
        assert!(tokens.contains(&"研发".to_string()));
        assert!(tokens.contains(&"方法".to_string()));
    }

    #[test]
    fn test_english_kept_as_is() {
        let tokens = tokenize("POC tool");
        assert!(tokens.contains(&"POC".to_string()));
        assert!(tokens.contains(&"tool".to_string()));
    }

    #[test]
    fn test_stopwords_removed() {
        let tokens = tokenize("的 了 是 在");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_short_tokens_removed() {
        let tokens = tokenize("a b c");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_mixed_cjk_english() {
        let tokens = tokenize("建立以 POC 为核心的快速探索范式");
        assert!(tokens.contains(&"POC".to_string()));
        assert!(tokens.contains(&"快速".to_string()));
        assert!(tokens.contains(&"探索".to_string()));
        assert!(tokens.contains(&"范式".to_string()));
    }

    #[test]
    fn test_no_duplicates() {
        let tokens = tokenize("研发 研发 研发");
        assert_eq!(tokens.iter().filter(|t| t == &"研发").count(), 1);
    }

    #[test]
    fn test_stopword_bigrams_removed() {
        let tokens = tokenize("我们");
        assert!(tokens.is_empty());
    }
}
