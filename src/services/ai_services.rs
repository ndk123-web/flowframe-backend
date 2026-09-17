use crate::dtos::ai_dto::{
    AiChatRequest, AiChatResponse, AiHistoryMessageItem, AiHistoryResponse, AiUsageDto,
};
use crate::models::ai_model::AiChatMessage;
use crate::repositories::ai_repository::AiRepository;
use crate::repositories::diagram_repository::DiagramRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;
use anyhow::{anyhow, Result};
use bson::oid::ObjectId;
use bson::DateTime;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

pub const MAX_AI_REQUESTS: i32 = 5;

#[derive(Clone)]
pub struct AiService {
    ai_repo: AiRepository,
    workspace_repo: WorkspaceRepository,
    diagram_repo: DiagramRepository,
    http_client: Client,
    gemini_api_key: String,
    gemini_model: String,
    openrouter_api_key: String,
}

impl AiService {
    pub fn new(
        ai_repo: AiRepository,
        workspace_repo: WorkspaceRepository,
        diagram_repo: DiagramRepository,
        gemini_api_key: String,
        gemini_model: String,
        openrouter_api_key: String,
    ) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_default();

        Self {
            ai_repo,
            workspace_repo,
            diagram_repo,
            http_client,
            gemini_api_key,
            gemini_model,
            openrouter_api_key,
        }
    }

    /// Process an incoming AI request (ASK, ANALYZE, or MODIFY)
    pub async fn process_chat(
        &self,
        user_id_str: &str,
        req: AiChatRequest,
    ) -> Result<AiChatResponse> {
        let user_id = ObjectId::parse_str(user_id_str)?;
        let ws_id = ObjectId::parse_str(&req.workspace_id)?;
        let diag_id = ObjectId::parse_str(&req.diagram_id)?;

        // 1. Verify workspace and diagram ownership
        let _ws = self
            .workspace_repo
            .find_by_id(&ws_id, &user_id)
            .await?
            .ok_or_else(|| anyhow!("Workspace not found or unauthorized"))?;

        let _diag = self
            .diagram_repo
            .find_by_id(&diag_id, &user_id)
            .await?
            .ok_or_else(|| anyhow!("Diagram not found or unauthorized"))?;

        // 2. Validate mode
        let normalized_mode = match req.mode.to_lowercase().as_str() {
            "ask" => "ask",
            "analyze" => "analyze",
            "modify" | "create" => "modify",
            _ => return Err(anyhow!("Invalid mode. Supported modes: 'ask', 'analyze', 'modify'")),
        };

        if req.message.trim().is_empty() {
            return Err(anyhow!("Message cannot be empty"));
        }

        // 3. Pre-check usage eligibility BEFORE calling Gemini (DO NOT increment yet!)
        let current_count = self.ai_repo.get_usage_count(&user_id).await?;
        if current_count >= MAX_AI_REQUESTS {
            return Err(anyhow!(
                "AI usage limit reached ({}/{}). Please upgrade for more requests.",
                MAX_AI_REQUESTS,
                MAX_AI_REQUESTS
            ));
        }

        // 4. Fetch recent isolated history for this workspace + diagram (last 6 messages)
        let history = self
            .ai_repo
            .find_recent_history(&ws_id, &diag_id, &user_id, 6)
            .await
            .unwrap_or_default();

        // 5. Construct FlowFrame system prompt & user payload
        let system_prompt = self.build_system_prompt();
        let prompt_payload = self.build_gemini_payload(&system_prompt, &history, &req, normalized_mode);

        if self.gemini_api_key.trim().is_empty() {
            return Err(anyhow!("Gemini API key is not configured on the server."));
        }

        // 6. Call Gemini API with Multi-Model Fallback and Retry Logic (handles 503 & 429)
        let text_output = self.call_gemini_with_fallback(&prompt_payload).await?;

        // 7. Parse structured AI response
        let parsed_json = self.extract_structured_json(&text_output, normalized_mode)?;

        let final_message = parsed_json["message"]
            .as_str()
            .unwrap_or("Here is the requested architecture guidance.")
            .to_string();

        let final_flow = parsed_json["flow"]
            .as_str()
            .filter(|s| !s.trim().is_empty() && s.trim() != "null")
            .map(|s| s.to_string());

        let final_explanation = parsed_json["explanation"]
            .as_str()
            .filter(|s| !s.trim().is_empty() && s.trim() != "null")
            .map(|s| s.to_string());

        let final_thought = parsed_json["thought_process"]
            .as_str()
            .filter(|s| !s.trim().is_empty() && s.trim() != "null")
            .map(|s| s.to_string());

        // 8. ATOMIC POST-INCREMENT: Increment user usage ONLY AFTER Gemini successfully responded!
        let usage = match self
            .ai_repo
            .check_and_increment_usage(&user_id, MAX_AI_REQUESTS)
            .await?
        {
            Some(u) => u,
            None => {
                return Err(anyhow!(
                    "AI usage limit reached ({}/{}). Please upgrade for more requests.",
                    MAX_AI_REQUESTS,
                    MAX_AI_REQUESTS
                ));
            }
        };

        let current_usage_dto = AiUsageDto {
            used: usage.request_count,
            limit: MAX_AI_REQUESTS,
            remaining: (MAX_AI_REQUESTS - usage.request_count).max(0),
        };

        // 9. Persist User Message & Assistant Message in isolated history
        let now = DateTime::now();

        let user_msg_record = AiChatMessage {
            id: None,
            user_id,
            workspace_id: ws_id,
            diagram_id: diag_id,
            role: "user".to_string(),
            mode: normalized_mode.to_string(),
            message: req.message.clone(),
            flow: None,
            explanation: None,
            thought_process: None,
            created_at: now,
        };
        let _ = self.ai_repo.save_message(&user_msg_record).await;

        let assistant_msg_record = AiChatMessage {
            id: None,
            user_id,
            workspace_id: ws_id,
            diagram_id: diag_id,
            role: "assistant".to_string(),
            mode: normalized_mode.to_string(),
            message: final_message.clone(),
            flow: final_flow.clone(),
            explanation: final_explanation.clone(),
            thought_process: final_thought.clone(),
            created_at: DateTime::now(),
        };
        let _ = self.ai_repo.save_message(&assistant_msg_record).await;

        Ok(AiChatResponse {
            mode: normalized_mode.to_string(),
            status: "success".to_string(),
            message: final_message,
            flow: final_flow,
            explanation: final_explanation,
            thought_process: final_thought,
            usage: current_usage_dto,
        })
    }

    /// Calls Gemini API with automatic retries and model fallback.
    /// If ALL Gemini models fail (e.g. 503 overload), falls back to OpenRouter.
    async fn call_gemini_with_fallback(&self, payload: &Value) -> Result<String> {
        let mut candidate_models = vec![
            self.gemini_model.clone(),
            "gemini-2.5-flash".to_string(),
            "gemini-2.0-flash".to_string(),
            "gemini-2.0-flash-lite".to_string(),
        ];
        // Deduplicate while preserving order
        let mut seen = std::collections::HashSet::new();
        candidate_models.retain(|m| seen.insert(m.clone()));

        let mut last_error = String::new();

        for model in &candidate_models {
            let api_url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
                model
            );

            // Up to 2 attempts per candidate model
            for attempt in 1..=2u64 {
                let response_res = self
                    .http_client
                    .post(&api_url)
                    .header("Content-Type", "application/json")
                    .header("X-goog-api-key", &self.gemini_api_key)
                    .json(payload)
                    .send()
                    .await;

                match response_res {
                    Ok(response) => {
                        let status = response.status();
                        if status.is_success() {
                            let gemini_resp: Value = response
                                .json()
                                .await
                                .map_err(|e| anyhow!("Failed to parse Gemini response: {}", e))?;

                            let text = gemini_resp["candidates"][0]["content"]["parts"][0]["text"]
                                .as_str()
                                .ok_or_else(|| anyhow!("Empty text received from Gemini"))?;

                            tracing::info!("Gemini model {} responded successfully", model);
                            return Ok(text.to_string());
                        }

                        let err_text = response.text().await.unwrap_or_default();
                        last_error = format!("Model {} returned ({}): {}", model, status, err_text);
                        tracing::warn!("Gemini model {} failed: {}", model, last_error);

                        // Retryable: 503 (Overloaded) or 429 (Rate Limit) — back off then try next model
                        if status.as_u16() == 503 || status.as_u16() == 429 {
                            if attempt < 2 {
                                sleep(Duration::from_millis(700 * attempt)).await;
                                continue;
                            }
                            // 2nd attempt also failed — try next Gemini model
                            break;
                        } else {
                            // Non-retryable HTTP error on this model, try next immediately
                            break;
                        }
                    }
                    Err(e) => {
                        last_error = format!("Network error connecting to Gemini model {}: {}", model, e);
                        tracing::warn!("{}", last_error);
                        sleep(Duration::from_millis(400)).await;
                    }
                }
            }
        }

        // ── All Gemini models exhausted ─────────────────────────────────────
        // Attempt OpenRouter as final fallback (does NOT count as a new credit deduction
        // since the outer process_chat only increments after this fn succeeds)
        tracing::warn!(
            "All Gemini models failed. Attempting OpenRouter fallback. Last error: {}",
            last_error
        );

        match self.call_openrouter_fallback(payload).await {
            Ok(text) => {
                tracing::info!("OpenRouter fallback succeeded");
                return Ok(text);
            }
            Err(or_err) => {
                tracing::error!("OpenRouter fallback also failed: {}", or_err);
            }
        }

        Err(anyhow!(
            "All AI providers temporarily unavailable (Gemini + OpenRouter). \
No credits were deducted. Please retry in a moment. Last Gemini error: {}",
            last_error
        ))
    }

    /// OpenRouter chat-completions fallback using a free model.
    /// Converts the Gemini-format payload to OpenAI-compatible messages.
    async fn call_openrouter_fallback(&self, gemini_payload: &Value) -> Result<String> {
        if self.openrouter_api_key.trim().is_empty() {
            return Err(anyhow!("OpenRouter API key not configured"));
        }

        // Build OpenAI-style messages from Gemini payload
        let mut messages: Vec<Value> = Vec::new();

        // System instruction
        if let Some(sys_parts) = gemini_payload["system_instruction"]["parts"].as_array() {
            let sys_text = sys_parts
                .iter()
                .filter_map(|p| p["text"].as_str())
                .collect::<Vec<_>>()
                .join("\n");
            if !sys_text.is_empty() {
                messages.push(json!({ "role": "system", "content": sys_text }));
            }
        }

        // Conversation history + current user turn
        if let Some(contents) = gemini_payload["contents"].as_array() {
            for turn in contents {
                let role = match turn["role"].as_str().unwrap_or("user") {
                    "model" => "assistant",
                    other => other,
                };
                let text = turn["parts"]
                    .as_array()
                    .and_then(|parts| parts.first())
                    .and_then(|p| p["text"].as_str())
                    .unwrap_or("");
                messages.push(json!({ "role": role, "content": text }));
            }
        }

        let openrouter_payload = json!({
            "model": "google/gemini-2.0-flash-exp:free",
            "messages": messages,
            "temperature": 0.3,
            "response_format": { "type": "json_object" }
        });

        let response = self
            .http_client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.openrouter_api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://flowframe.app")
            .header("X-Title", "FlowFrame Relay AI")
            .json(&openrouter_payload)
            .send()
            .await
            .map_err(|e| anyhow!("OpenRouter network error: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let err_body = response.text().await.unwrap_or_default();
            return Err(anyhow!("OpenRouter returned {}: {}", status, err_body));
        }

        let or_resp: Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse OpenRouter response: {}", e))?;

        let text = or_resp["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Empty content from OpenRouter response"))?;

        Ok(text.to_string())
    }

    /// Read usage statistics for a user
    pub async fn get_usage(&self, user_id_str: &str) -> Result<AiUsageDto> {
        let user_id = ObjectId::parse_str(user_id_str)?;
        let count = self.ai_repo.get_usage_count(&user_id).await?;
        Ok(AiUsageDto {
            used: count,
            limit: MAX_AI_REQUESTS,
            remaining: (MAX_AI_REQUESTS - count).max(0),
        })
    }

    /// Reset usage statistics for a user (e.g. after transient failures)
    pub async fn reset_usage(&self, user_id_str: &str) -> Result<()> {
        let user_id = ObjectId::parse_str(user_id_str)?;
        self.ai_repo.reset_usage(&user_id).await
    }

    /// Get isolated history for a diagram
    pub async fn get_history(
        &self,
        user_id_str: &str,
        workspace_id_str: &str,
        diagram_id_str: &str,
    ) -> Result<AiHistoryResponse> {
        let user_id = ObjectId::parse_str(user_id_str)?;
        let ws_id = ObjectId::parse_str(workspace_id_str)?;
        let diag_id = ObjectId::parse_str(diagram_id_str)?;

        let messages = self
            .ai_repo
            .find_recent_history(&ws_id, &diag_id, &user_id, 50)
            .await?;

        let items: Vec<AiHistoryMessageItem> = messages
            .into_iter()
            .map(|m| AiHistoryMessageItem {
                id: m.id.map(|id| id.to_hex()).unwrap_or_default(),
                role: m.role,
                mode: m.mode,
                message: m.message,
                flow: m.flow,
                explanation: m.explanation,
                thought_process: m.thought_process,
                created_at: m.created_at.to_string(),
            })
            .collect();

        let count = self.ai_repo.get_usage_count(&user_id).await?;
        let usage = AiUsageDto {
            used: count,
            limit: MAX_AI_REQUESTS,
            remaining: (MAX_AI_REQUESTS - count).max(0),
        };

        Ok(AiHistoryResponse {
            messages: items,
            usage,
        })
    }

    
    pub async fn get_user_usage(&self, user_id_str: &str) -> Result<AiUsageDto> {
        self.get_usage(user_id_str).await
    }

    pub async fn get_chat_history(
        &self,
        user_id_str: &str,
        workspace_id_str: &str,
        diagram_id_str: &str,
    ) -> Result<AiHistoryResponse> {
        self.get_history(user_id_str, workspace_id_str, diagram_id_str).await
    }

    fn build_system_prompt(&self) -> String {
        r#"You are Relay, the elite AI Systems Architect and Distributed Engine Copilot for FlowFrame.
FlowFrame simulates distributed systems (Clients, Load Balancers, Servers, Redis Caches, PostgreSQL, Message Queues, PubSub, Gateways).
Architectures in FlowFrame are defined in a domain-specific language (DSL).

==================================================
FLOWFRAME DSL SYNTAX SPECIFICATION
==================================================

1. NODE DEFINITIONS:
define <TYPE> <ID> {
  x: <number>,
  y: <number>,
  label: "<string>"
  // Optional parameters depending on node type
}

VALID NODE TYPES:
- CLIENT: Originates HTTP requests
  Optional properties:
  requests: [ { endpoint: "/api/path", allowedMethods: ["GET", "POST"], key: "cache:key" } ]
- SERVER: Computes logic and connects to databases
  Optional properties:
  capacity: <number> (e.g. 100),
  acceptedEndpoints: [ { endpoint: "/api/path", allowedMethod: ["GET", "POST"] } ]
- LOADBALANCER: Balances traffic
  strategy: "ROUND_ROBIN" | "LEAST_CONNECTIONS"
- GATEWAY: Ingress routing gateway
  strategy: "ROUND_ROBIN"
- REDIS: In-memory cache layer
  data: [ { key: "item:1", value: "cached payload" } ]
- POSTGRES: Relational ACID database
  data: [ { key: "item:1", value: "persistent row" } ]
- MESSAGEQUEUE: Asynchronous buffer
  queueSize: <number>,
  processingType: "FIFO"
- PUBSUB: Event fanout broker
  topic: "<topic.name>"

2. CONNECTIONS:
connect <SOURCE_ID> -> <TARGET_ID>

==================================================
THE THREE MODES:
==================================================

1. "ask": Conceptual questions, protocol explanation, or topology explanation.
   - You MUST NOT return any FlowFrame DSL in the "flow" field. Keep "flow": null.
   - Provide a clear, insightful explanation of distributed system mechanics.

2. "analyze": Architecture health check, bottlenecks, single points of failure.
   - You MUST NOT mutate the canvas. Keep "flow": null.
   - Identify unrouted components, missing load balancers, database connection bottlenecks, failure dynamics.

3. "modify": Propose a complete, compilable FlowFrame DSL architecture.
   - You MUST generate valid, syntax-clean FlowFrame DSL in the "flow" field.
   - Provide clean (x, y) coordinates with horizontal flow (e.g. x: 80, x: 380, x: 680, x: 980).
   - "Fixing" an architecture is treated as a modify intent.

==================================================
OUTPUT FORMAT:
==================================================
You MUST respond with a single valid, raw JSON object:
{
  "mode": "ask" | "analyze" | "modify",
  "message": "High-level summary of the answer or changes",
  "explanation": "Detailed technical explanation formatted in markdown",
  "flow": "Full FlowFrame DSL code string (ONLY for modify mode, otherwise null)",
  "thought_process": "Step-by-step reasoning trace"
}
"#.to_string()
    }

    fn build_gemini_payload(
        &self,
        system_prompt: &str,
        history: &[AiChatMessage],
        req: &AiChatRequest,
        mode: &str,
    ) -> Value {
        let mut contents = Vec::new();

        // 1. History
        for msg in history {
            let role = if msg.role == "assistant" { "model" } else { "user" };
            contents.push(json!({
                "role": role,
                "parts": [{ "text": &msg.message }]
            }));
        }

        // 2. Current User Turn with context
        let mut user_text = format!("Mode: {}
User Request: {}
", mode, req.message);

        if let Some(ctx) = &req.context {
            user_text.push_str("
--- CURRENT CANVAS CONTEXT ---
");
            if let Some(node_cnt) = ctx.node_count {
                user_text.push_str(&format!("Total Components: {}
", node_cnt));
            }
            if let Some(edge_cnt) = ctx.edge_count {
                user_text.push_str(&format!("Total Connections: {}
", edge_cnt));
            }
            if let Some(sel) = &ctx.selected_node_id {
                user_text.push_str(&format!("Selected Component: {}
", sel));
            }
            if let Some(dsl) = &ctx.dsl {
                user_text.push_str(&format!("
--- ACTIVE CANVAS DSL ---
{}
", dsl));
            }
        }

        contents.push(json!({
            "role": "user",
            "parts": [{ "text": user_text }]
        }));

        json!({
            "system_instruction": {
                "parts": [{ "text": system_prompt }]
            },
            "contents": contents,
            "generationConfig": {
                "temperature": if req.think { 0.2 } else { 0.3 },
                "response_mime_type": "application/json"
            }
        })
    }

    fn extract_structured_json(&self, raw_text: &str, mode: &str) -> Result<Value> {
        let trimmed = raw_text.trim();

        // Attempt direct JSON parse
        if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
            return Ok(v);
        }

        // Strip markdown ```json ... ```
        if let Some(start) = trimmed.find("```json") {
            let slice = &trimmed[start + 7..];
            if let Some(end) = slice.find("```") {
                let code_part = slice[..end].trim();
                if let Ok(v) = serde_json::from_str::<Value>(code_part) {
                    return Ok(v);
                }
            }
        } else if let Some(start) = trimmed.find("```") {
            let slice = &trimmed[start + 3..];
            if let Some(end) = slice.find("```") {
                let code_part = slice[..end].trim();
                if let Ok(v) = serde_json::from_str::<Value>(code_part) {
                    return Ok(v);
                }
            }
        }

        // Fallback structured value
        Ok(json!({
            "mode": mode,
            "message": trimmed,
            "explanation": trimmed,
            "flow": null,
            "thought_process": null
        }))
    }
}
