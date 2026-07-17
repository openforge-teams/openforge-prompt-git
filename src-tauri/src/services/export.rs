use crate::error::AppResult;
use crate::models::{ExportCodeRequest, InferenceParams};
use crate::services::version::get_version_by_hash;
use chrono::Utc;
use rusqlite::Connection;

pub fn export_code_snippet(req: ExportCodeRequest) -> AppResult<String> {
    let sys = escape_for_lang(&req.system_prompt, &req.language);
    let user = escape_for_lang(&req.user_prompt, &req.language);
    let model = &req.model_name;
    let temp = req.params.temperature;
    let max_tokens = req.params.max_tokens;
    let top_p = req.params.top_p;

    let code = match req.language.to_lowercase().as_str() {
        "python" => format!(
            r#"from openai import OpenAI

client = OpenAI(api_key="your_api_key")

response = client.chat.completions.create(
    model="{model}",
    messages=[
        {{"role": "system", "content": "{sys}"}},
        {{"role": "user", "content": "{user}"}}
    ],
    temperature={temp},
    max_tokens={max_tokens},
    top_p={top_p}
)

print(response.choices[0].message.content)
"#
        ),
        "javascript" | "js" => format!(
            r#"import OpenAI from "openai";

const client = new OpenAI({{ apiKey: process.env.OPENAI_API_KEY }});

const response = await client.chat.completions.create({{
  model: "{model}",
  messages: [
    {{ role: "system", content: `{sys}` }},
    {{ role: "user", content: `{user}` }}
  ],
  temperature: {temp},
  max_tokens: {max_tokens},
  top_p: {top_p}
}});

console.log(response.choices[0].message.content);
"#
        ),
        "typescript" | "ts" => format!(
            r#"import OpenAI from "openai";

const client = new OpenAI({{ apiKey: process.env.OPENAI_API_KEY }});

const response = await client.chat.completions.create({{
  model: "{model}",
  messages: [
    {{ role: "system", content: `{sys}` }},
    {{ role: "user", content: `{user}` }}
  ],
  temperature: {temp},
  max_tokens: {max_tokens},
  top_p: {top_p}
}});

console.log(response.choices[0].message.content);
"#
        ),
        "go" => format!(
            r#"package main

import (
  "context"
  "fmt"
  "os"

  openai "github.com/sashabaranov/go-openai"
)

func main() {{
  client := openai.NewClient(os.Getenv("OPENAI_API_KEY"))
  resp, err := client.CreateChatCompletion(context.Background(), openai.ChatCompletionRequest{{
    Model: "{model}",
    Messages: []openai.ChatCompletionMessage{{
      {{Role: openai.ChatMessageRoleSystem, Content: `{sys}`}},
      {{Role: openai.ChatMessageRoleUser, Content: `{user}`}},
    }},
    Temperature: float32({temp}),
    MaxTokens: {max_tokens},
    TopP: float32({top_p}),
  }})
  if err != nil {{
    panic(err)
  }}
  fmt.Println(resp.Choices[0].Message.Content)
}}
"#
        ),
        "java" => format!(
            r#"// Requires OpenAI Java SDK
var client = OpenAIOkHttpClient.builder()
    .apiKey(System.getenv("OPENAI_API_KEY"))
    .build();

var params = ChatCompletionCreateParams.builder()
    .model("{model}")
    .addSystemMessage("{sys}")
    .addUserMessage("{user}")
    .temperature({temp})
    .maxCompletionTokens({max_tokens})
    .topP({top_p})
    .build();

var completion = client.chat().completions().create(params);
System.out.println(completion.choices().get(0).message().content().orElse(""));
"#
        ),
        "rust" => format!(
            r#"// Requires async-openai crate
use async_openai::{{Client, types::{{CreateChatCompletionRequestArgs, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs}}}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    let client = Client::new();
    let request = CreateChatCompletionRequestArgs::default()
        .model("{model}")
        .messages([
            ChatCompletionRequestSystemMessageArgs::default().content("{sys}").build()?.into(),
            ChatCompletionRequestUserMessageArgs::default().content("{user}").build()?.into(),
        ])
        .temperature({temp} as f32)
        .max_tokens({max_tokens}u16)
        .top_p({top_p} as f32)
        .build()?;
    let response = client.chat().create(request).await?;
    println!("{{}}", response.choices[0].message.content.as_deref().unwrap_or(""));
    Ok(())
}}
"#
        ),
        "curl" => format!(
            r#"curl https://api.openai.com/v1/chat/completions \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{{
    "model": "{model}",
    "messages": [
      {{"role": "system", "content": "{sys}"}},
      {{"role": "user", "content": "{user}"}}
    ],
    "temperature": {temp},
    "max_tokens": {max_tokens},
    "top_p": {top_p}
  }}'
"#
        ),
        other => {
            return Err(crate::error::AppError::msg(format!(
                "unsupported language: {other}"
            )));
        }
    };
    Ok(code)
}

fn escape_for_lang(s: &str, lang: &str) -> String {
    match lang.to_lowercase().as_str() {
        "python" | "java" | "curl" => s
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n"),
        _ => s.replace('`', "\\`"),
    }
}

pub fn export_json(
    conn: &Connection,
    file_id: i64,
    version_hash: &str,
    model_name: Option<&str>,
    params: Option<InferenceParams>,
) -> AppResult<String> {
    let version = get_version_by_hash(conn, version_hash)?;
    let name: String = conn.query_row(
        "SELECT name FROM prompt_files WHERE id = ?1",
        rusqlite::params![file_id],
        |row| row.get(0),
    )?;
    let variables = crate::services::template::extract_variables(&version.user_prompt);
    let params = params.unwrap_or_default();
    let payload = serde_json::json!({
        "prompt_info": {
            "id": file_id,
            "name": name,
            "version": format!("v{}", version.id),
            "version_hash": version.version_hash,
            "system_prompt": version.system_prompt,
            "user_prompt": version.user_prompt,
            "commit_message": version.commit_message,
            "remark": version.remark
        },
        "model_params": {
            "model": model_name.unwrap_or("gpt-4o-mini"),
            "temperature": params.temperature,
            "max_tokens": params.max_tokens,
            "top_p": params.top_p,
            "frequency_penalty": params.frequency_penalty
        },
        "variables": variables,
        "export_time": Utc::now().to_rfc3339()
    });
    Ok(serde_json::to_string_pretty(&payload)?)
}

pub fn export_yaml(
    conn: &Connection,
    file_id: i64,
    version_hash: &str,
) -> AppResult<String> {
    let json = export_json(conn, file_id, version_hash, None, None)?;
    let value: serde_json::Value = serde_json::from_str(&json)?;
    Ok(serde_yaml::to_string(&value)?)
}

pub fn export_markdown_report(
    prompt_name: &str,
    system_prompt: &str,
    user_prompt: &str,
    model_params: &InferenceParams,
    results: &[(String, String, Option<f64>, Option<i64>, Option<String>)],
) -> String {
    let mut md = String::new();
    md.push_str(&format!("# Prompt Compare Report: {prompt_name}\n\n"));
    md.push_str(&format!("Exported at: {}\n\n", Utc::now().to_rfc3339()));
    md.push_str("## Prompt\n\n");
    if !system_prompt.is_empty() {
        md.push_str("### System\n\n```\n");
        md.push_str(system_prompt);
        md.push_str("\n```\n\n");
    }
    md.push_str("### User\n\n```\n");
    md.push_str(user_prompt);
    md.push_str("\n```\n\n");
    md.push_str("## Parameters\n\n");
    md.push_str(&format!(
        "- temperature: {}\n- max_tokens: {}\n- top_p: {}\n\n",
        model_params.temperature, model_params.max_tokens, model_params.top_p
    ));
    md.push_str("## Results\n\n");
    md.push_str("| Model | Score | Latency (ms) | Evaluation | Output |\n");
    md.push_str("|---|---:|---:|---|---|\n");
    for (model, output, score, latency, eval) in results {
        let out = output.replace('|', "\\|").replace('\n', "<br>");
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            model,
            score.map(|s| format!("{s:.1}")).unwrap_or_else(|| "-".into()),
            latency.map(|l| l.to_string()).unwrap_or_else(|| "-".into()),
            eval.clone().unwrap_or_else(|| "-".into()).replace('|', "\\|"),
            out
        ));
    }
    md
}

pub fn export_plain_prompt(system_prompt: &str, user_prompt: &str) -> String {
    if system_prompt.is_empty() {
        user_prompt.to_string()
    } else {
        format!("[SYSTEM]\n{system_prompt}\n\n[USER]\n{user_prompt}")
    }
}
