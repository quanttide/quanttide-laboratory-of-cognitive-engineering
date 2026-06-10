use std::fs;

use quanttide_agent::message::Message;
use quanttide_agent::llm::{LLM, CompleteOptions};

fn main() -> Result<(), String> {
    let input = fs::read_to_string("data/input.md")
        .map_err(|e| format!("读取输入文件失败: {}", e))?;

    let text = extract_segment(&input);

    println!("=== 输入文本（{} 字）===\n{}\n", text.chars().count(), text);

    let llm = LLM::default();

    let control_prompt = format!(
        "以下是一段原始思考日志，请提取关键信息，用简洁的列表输出：\n\n{}",
        text
    );

    let experimental_prompt = format!(
        "以下是一段原始思考日志，请按认知工程框架提取结构化信息：\n\
         \n\
         1. Mental Model（心智模型）：这段思考反映了什么认知模式？\n\
         2. Schematic（图示）：可以抽象出什么因果关系或决策框架？\n\
         3. Situation（情境）：当前面对的具体情境是什么？请从 agenda（目标）、ecology（环境）、frame（认知框架）、dynamics（动态）四个维度描述。\n\
         4. Intent（意图）：这段思考催生了什么明确的意图？包括优先级、风险和触发条件。\n\
         \n\
         原始日志：\n\n{}",
        text
    );

    let options = CompleteOptions::default();

    println!("=== 对照组：自由提取 ===\n");
    let control_resp = llm.complete(&[
        Message::new("user", &control_prompt),
    ], options).map_err(|e| format!("LLM 调用失败: {}", e))?;
    println!("{}\n", control_resp.content);

    println!("=== 实验组：认知工程提取 ===\n");
    let experimental_resp = llm.complete(&[
        Message::new("user", &experimental_prompt),
    ], CompleteOptions::default()).map_err(|e| format!("LLM 调用失败: {}", e))?;
    println!("{}\n", experimental_resp.content);

    println!("=== 对比 ===");
    println!("对照组长度: {} 字", control_resp.content.chars().count());
    println!("实验组长度: {} 字", experimental_resp.content.chars().count());
    println!("对照组 finish_reason: {}", control_resp.finish_reason);
    println!("实验组 finish_reason: {}", experimental_resp.finish_reason);

    Ok(())
}

fn extract_segment(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let start = lines.iter().position(|l| l.contains("备份")).unwrap_or(12);
    let end = lines.len().min(start + 15);
    lines[start..end].join("\n")
}
