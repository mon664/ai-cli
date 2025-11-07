use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::env;

/// AI 연동 모듈
/// 로컬(Ollama)과 원격(OpenAI, Anthropic) AI 모델을 지원

/// AI 백엔드 종류
#[derive(Debug, Clone)]
pub enum AIBackend {
    Local { model: String, url: String },
    OpenAI { model: String, api_key: String },
    Anthropic { model: String, api_key: String },
}

/// AI 응답 구조체
#[derive(Debug, Deserialize)]
pub struct AIResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
}

/// 토큰 사용량 정보
#[derive(Debug, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// OpenAI API 응답 구조체
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// 커밋 메시지 생성을 위한 프롬프트 생성
pub fn create_commit_prompt(diff: &str, extra_context: Option<&str>) -> String {
    let context_section = if let Some(context) = extra_context {
        format!("\nADDITIONAL CONTEXT:\n{}\n", context)
    } else {
        String::new()
    };

    format!(
        r#"SYSTEM:
You are an expert-level Git assistant specialized in writing Conventional Commit messages.
Your task is to analyze the provided 'git diff' output and generate a concise, accurate, and properly formatted commit message.

RULES:
1. You MUST follow the Conventional Commits specification strictly.
2. The output MUST be only the commit message, starting with `<type>[optional scope]: <description>`.
3. Choose the correct `<type>` from: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`.
4. The `<description>` must be lowercase, start with an imperative verb (e.g., "add", "fix", "update"), and be no more than 72 characters.
5. If the changes are significant, provide a body explaining the "what" and "why" separated by a blank line.
6. If there are breaking changes, add a `BREAKING CHANGE:` footer.
7. Consider the impact on users and other developers.
8. Be specific but concise - avoid generic messages like "update files".

TYPE GUIDELINES:
- feat: new feature for the user, not a new feature for build process
- fix: bug fix for the user, not a fix to a build script
- docs: documentation changes only
- style: formatting, missing semi colons, etc; no code logic change
- refactor: refactoring production code, eg. renaming a variable
- test: adding tests, refactoring test; no production code change
- build: changes to build system or external dependencies
- ci: changes to CI configuration files and scripts
- chore: updating deps, updating build config, etc; no production code change

{}Analyze the following diff of staged changes and generate only the commit message:

```diff
{}
```

COMMIT_MESSAGE:"#,
        context_section, diff
    )
}

/// 코드 변경 사항 설명을 위한 프롬프트 생성
pub fn create_explain_prompt(diff: &str, detailed: bool) -> String {
    if detailed {
        format!(
            r#"SYSTEM:
You are an expert software engineer tasked with explaining code changes in detail.
Analyze the provided diff and provide a comprehensive explanation including:

1. High-level summary of what changed
2. Technical details of the implementation changes
3. Reasoning behind the changes (why these changes were made)
4. Potential impact on the codebase
5. Any breaking changes or migration requirements

Provide your response in well-structured markdown with clear sections.

DIFF TO ANALYZE:
```diff
{}
```

EXPLANATION:"#,
            diff
        )
    } else {
        format!(
            r#"SYSTEM:
You are a software engineer helping developers understand code changes quickly.
Analyze the provided diff and provide a concise, clear explanation in 2-3 paragraphs covering:
- What changed (high level)
- Why it was changed
- Main impact or benefit

Keep it technical but accessible.

DIFF TO ANALYZE:
```diff
{}
```

EXPLANATION:"#,
            diff
        )
    }
}

/// 로컬 Ollama를 사용하여 커밋 메시지 생성
pub async fn generate_commit_local(diff: &str, extra_context: Option<&str>) -> Result<AIResponse> {
    let model = env::var("AI_CLI_LOCAL_MODEL").unwrap_or_else(|_| "gemma2:9b".to_string());
    let url = env::var("AI_CLI_OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());

    let prompt = create_commit_prompt(diff, extra_context);

    // Ollama API 클라이언트 생성
    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "options": {
            "temperature": 0.3,
            "top_p": 0.9,
            "max_tokens": 150
        }
    });

    let response = client
        .post(&format!("{}/api/generate", url))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to connect to Ollama at {}: {}", url, e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(anyhow!("Ollama API error: {}", error_text));
    }

    #[derive(Deserialize)]
    struct OllamaResponse {
        response: String,
        done: bool,
        total_duration: Option<u64>,
        prompt_eval_count: Option<u32>,
        eval_count: Option<u32>,
    }

    let ollama_response: OllamaResponse = response.json().await
        .map_err(|e| anyhow!("Failed to parse Ollama response: {}", e))?;

    let content = ollama_response.response.trim().to_string();

    // Conventional Commit 형식 검증 및 정제
    let refined_content = refine_conventional_commit(&content);

    Ok(AIResponse {
        content: refined_content,
        model,
        usage: Some(TokenUsage {
            prompt_tokens: ollama_response.prompt_eval_count.unwrap_or(0),
            completion_tokens: ollama_response.eval_count.unwrap_or(0),
            total_tokens: ollama_response.prompt_eval_count.unwrap_or(0) + ollama_response.eval_count.unwrap_or(0),
        }),
    })
}

/// OpenAI API를 사용하여 커밋 메시지 생성
pub async fn generate_commit_openai(diff: &str, extra_context: Option<&str>) -> Result<AIResponse> {
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow!("OPENAI_API_KEY environment variable is not set"))?;

    let model = env::var("AI_CLI_OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
    let prompt = create_commit_prompt(diff, extra_context);

    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": "You are an expert Git assistant. Generate conventional commit messages only, without any additional text or explanations."
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "temperature": 0.3,
        "max_tokens": 150,
        "top_p": 0.9
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to call OpenAI API: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(anyhow!("OpenAI API error: {}", error_text));
    }

    let openai_response: OpenAIResponse = response.json().await
        .map_err(|e| anyhow!("Failed to parse OpenAI response: {}", e))?;

    let content = openai_response.choices
        .get(0)
        .and_then(|choice| Some(choice.message.content.trim().to_string()))
        .ok_or_else(|| anyhow!("No response from OpenAI API"))?;

    // Conventional Commit 형식 검증 및 정제
    let refined_content = refine_conventional_commit(&content);

    Ok(AIResponse {
        content: refined_content,
        model,
        usage: Some(TokenUsage {
            prompt_tokens: openai_response.usage.prompt_tokens,
            completion_tokens: openai_response.usage.completion_tokens,
            total_tokens: openai_response.usage.total_tokens,
        }),
    })
}

/// 변경 사항 설명 생성
pub async fn generate_explanation(diff: &str, detailed: bool, backend: &AIBackend) -> Result<AIResponse> {
    let prompt = create_explain_prompt(diff, detailed);

    match backend {
        AIBackend::Local { model, url } => {
            let client = reqwest::Client::new();

            let request_body = serde_json::json!({
                "model": model,
                "prompt": prompt,
                "stream": false,
                "options": {
                    "temperature": 0.5,
                    "top_p": 0.9,
                    "max_tokens": if detailed { 500 } else { 200 }
                }
            });

            let response = client
                .post(&format!("{}/api/generate", url))
                .json(&request_body)
                .send()
                .await
                .map_err(|e| anyhow!("Failed to connect to Ollama at {}: {}", url, e))?;

            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_default();
                return Err(anyhow!("Ollama API error: {}", error_text));
            }

            #[derive(Deserialize)]
            struct OllamaResponse {
                response: String,
                eval_count: Option<u32>,
                prompt_eval_count: Option<u32>,
            }

            let ollama_response: OllamaResponse = response.json().await
                .map_err(|e| anyhow!("Failed to parse Ollama response: {}", e))?;

            Ok(AIResponse {
                content: ollama_response.response.trim().to_string(),
                model: model.clone(),
                usage: Some(TokenUsage {
                    prompt_tokens: ollama_response.prompt_eval_count.unwrap_or(0),
                    completion_tokens: ollama_response.eval_count.unwrap_or(0),
                    total_tokens: ollama_response.prompt_eval_count.unwrap_or(0) + ollama_response.eval_count.unwrap_or(0),
                }),
            })
        }
        AIBackend::OpenAI { model, api_key } => {
            let client = reqwest::Client::new();

            let request_body = serde_json::json!({
                "model": model,
                "messages": [
                    {
                        "role": "system",
                        "content": "You are an expert software engineer. Analyze code changes and provide clear, concise explanations."
                    },
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "temperature": 0.5,
                "max_tokens": if detailed { 500 } else { 200 }
            });

            let response = client
                .post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await
                .map_err(|e| anyhow!("Failed to call OpenAI API: {}", e))?;

            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_default();
                return Err(anyhow!("OpenAI API error: {}", error_text));
            }

            let openai_response: OpenAIResponse = response.json().await
                .map_err(|e| anyhow!("Failed to parse OpenAI response: {}", e))?;

            let content = openai_response.choices
                .get(0)
                .and_then(|choice| Some(choice.message.content.trim().to_string()))
                .ok_or_else(|| anyhow!("No response from OpenAI API"))?;

            Ok(AIResponse {
                content,
                model: model.clone(),
                usage: Some(TokenUsage {
                    prompt_tokens: openai_response.usage.prompt_tokens,
                    completion_tokens: openai_response.usage.completion_tokens,
                    total_tokens: openai_response.usage.total_tokens,
                }),
            })
        }
        AIBackend::Anthropic { model, api_key } => {
            let client = reqwest::Client::new();

            let request_body = serde_json::json!({
                "model": model,
                "max_tokens": if detailed { 500 } else { 200 },
                "temperature": 0.5,
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ]
            });

            let response = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await
                .map_err(|e| anyhow!("Failed to call Anthropic API: {}", e))?;

            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_default();
                return Err(anyhow!("Anthropic API error: {}", error_text));
            }

            #[derive(Deserialize)]
            struct AnthropicResponse {
                content: Vec<AnthropicContent>,
                usage: AnthropicUsage,
            }

            #[derive(Deserialize)]
            struct AnthropicContent {
                type_: String,
                text: String,
            }

            #[derive(Deserialize)]
            struct AnthropicUsage {
                input_tokens: u32,
                output_tokens: u32,
            }

            let anthropic_response: AnthropicResponse = response.json().await
                .map_err(|e| anyhow!("Failed to parse Anthropic response: {}", e))?;

            let content = anthropic_response.content
                .get(0)
                .and_then(|c| Some(c.text.clone()))
                .ok_or_else(|| anyhow!("No content in Anthropic response"))?;

            Ok(AIResponse {
                content: content.trim().to_string(),
                model: model.clone(),
                usage: Some(TokenUsage {
                    prompt_tokens: anthropic_response.usage.input_tokens,
                    completion_tokens: anthropic_response.usage.output_tokens,
                    total_tokens: anthropic_response.usage.input_tokens + anthropic_response.usage.output_tokens,
                }),
            })
        }
    }
}

/// Conventional Commit 형식 검증 및 정제
fn refine_conventional_commit(message: &str) -> String {
    let mut refined = message.trim().to_string();

    // 불필요한 접두사/접미사 제거
    let prefixes_to_remove = [
        "Commit message:",
        "Here's the commit message:",
        "The commit message is:",
        "COMMIT_MESSAGE:",
        "```",
        "Conventional commit:",
    ];

    for prefix in &prefixes_to_remove {
        if refined.starts_with(prefix) {
            refined = refined.strip_prefix(prefix).unwrap_or(&refined).trim().to_string();
        }
    }

    // 코드 블록 제거
    if refined.starts_with("```") {
        let lines: Vec<&str> = refined.lines().collect();
        if lines.len() > 2 {
            refined = lines[1..lines.len()-1].join("\n");
        }
    }

    // 따옴표 제거
    if refined.starts_with('"') && refined.ends_with('"') {
        refined = refined[1..refined.len()-1].to_string();
    }

    // Conventional Commit 타입 확인
    let types = ["feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert"];
    let has_valid_type = types.iter().any(|&t| refined.starts_with(&format!("{}:", t)) ||
                                       refined.starts_with(&format!("{}(", t)));

    // 유효한 타입이 없으면 기본 타입 추가
    if !has_valid_type {
        if refined.contains("add") || refined.contains("new") || refined.contains("implement") {
            refined = format!("feat: {}", refined);
        } else if refined.contains("fix") || refined.contains("bug") || refined.contains("error") {
            refined = format!("fix: {}", refined);
        } else if refined.contains("update") || refined.contains("change") {
            refined = format!("refactor: {}", refined);
        } else if refined.contains("test") {
            refined = format!("test: {}", refined);
        } else if refined.contains("doc") {
            refined = format!("docs: {}", refined);
        } else {
            refined = format!("chore: {}", refined);
        }
    }

    // 길이 제한 (72자)
    if let Some(first_line) = refined.lines().next() {
        if first_line.len() > 72 {
            let trimmed = &first_line[..72.min(first_line.len())];
            refined = refined.replacen(first_line, trimmed, 1);
        }
    }

    refined
}

/// 커밋 메시지 생성 (메인 진입점)
pub async fn generate_commit_message(diff: &str) -> Result<String> {
    // 기본적으로 로컬 모델 시도
    match generate_commit_local(diff, None).await {
        Ok(response) => Ok(response.content),
        Err(e) => {
            tracing::warn!("Local model failed: {}, trying OpenAI", e);

            // OpenAI 폴백
            match generate_commit_openai(diff, None).await {
                Ok(response) => Ok(response.content),
                Err(e) => {
                    tracing::error!("All AI backends failed: {}", e);
                    Err(anyhow!("Failed to generate commit message with any available AI backend"))
                }
            }
        }
    }
}

/// 설정에서 AI 백엔드 결정
pub fn get_ai_backend(model_preference: &str) -> Result<AIBackend> {
    match model_preference {
        "local" => {
            let model = env::var("AI_CLI_LOCAL_MODEL").unwrap_or_else(|_| "gemma2:9b".to_string());
            let url = env::var("AI_CLI_OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
            Ok(AIBackend::Local { model, url })
        }
        "openai" => {
            let api_key = env::var("OPENAI_API_KEY")
                .map_err(|_| anyhow!("OPENAI_API_KEY not set"))?;
            let model = env::var("AI_CLI_OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
            Ok(AIBackend::OpenAI { model, api_key })
        }
        "anthropic" => {
            let api_key = env::var("ANTHROPIC_API_KEY")
                .map_err(|_| anyhow!("ANTHROPIC_API_KEY not set"))?;
            let model = env::var("AI_CLI_ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string());
            Ok(AIBackend::Anthropic { model, api_key })
        }
        _ => Err(anyhow!("Unsupported model: {}. Use 'local', 'openai', or 'anthropic'", model_preference))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_commit_prompt() {
        let diff = "+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"Hello, world!\");\n }\n";
        let prompt = create_commit_prompt(diff, None);

        assert!(prompt.contains("Conventional Commits"));
        assert!(prompt.contains(diff));
    }

    #[test]
    fn test_create_explain_prompt() {
        let diff = "+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"Hello, world!\");\n }\n";
        let prompt = create_explain_prompt(diff, false);

        assert!(prompt.contains("software engineer"));
        assert!(prompt.contains(diff));
    }
}