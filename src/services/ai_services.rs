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
            "gemini-flash-latest".to_string(),
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

    /// OpenRouter chat-completions fallback across multiple free models.
    /// Converts Gemini-format payload to OpenAI-compatible format and tries models sequentially.
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

        // Ordered fallback models on OpenRouter (free tier)
        let openrouter_models = [
            "inclusionai/ling-3.0-flash-vl:free",
            "nex-agi/nex-n2.5-mini:free",
            "nvidia/nemotron-3.5-lightning:free",
            "poolside/laguna-s-2.1:free",
        ];

        let mut last_or_err = String::new();

        for model in &openrouter_models {
            let openrouter_payload = json!({
                "model": model,
                "messages": messages,
                "temperature": 0.3,
            });

            tracing::info!("Attempting OpenRouter fallback model: {}", model);

            let response_res = self
                .http_client
                .post("https://openrouter.ai/api/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", self.openrouter_api_key))
                .header("Content-Type", "application/json")
                .header("HTTP-Referer", "https://flowframe.app")
                .header("X-Title", "FlowFrame Relay AI")
                .json(&openrouter_payload)
                .send()
                .await;

            match response_res {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        let or_resp: Value = match response.json().await {
                            Ok(v) => v,
                            Err(e) => {
                                last_or_err = format!("OpenRouter model {} JSON parse error: {}", model, e);
                                tracing::warn!("{}", last_or_err);
                                continue;
                            }
                        };

                        if let Some(text) = or_resp["choices"][0]["message"]["content"].as_str() {
                            let trimmed = text.trim();
                            if !trimmed.is_empty() {
                                tracing::info!("OpenRouter model {} succeeded", model);
                                return Ok(trimmed.to_string());
                            }
                        }
                        last_or_err = format!("Empty content returned from OpenRouter model {}", model);
                        tracing::warn!("{}", last_or_err);
                    } else {
                        let err_body = response.text().await.unwrap_or_default();
                        last_or_err = format!("Model {} returned ({}): {}", model, status, err_body);
                        tracing::warn!("OpenRouter model {} failed: {}", model, last_or_err);
                    }
                }
                Err(e) => {
                    last_or_err = format!("Network error with OpenRouter model {}: {}", model, e);
                    tracing::warn!("{}", last_or_err);
                }
            }
        }

        Err(anyhow!(
            "All OpenRouter fallback models failed. Last error: {}",
            last_or_err
        ))
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
FlowFrame simulates distributed systems (Clients, Servers, Gateways, Load Balancers, Redis, PostgreSQL, Message Queues, PubSub).
Architectures in FlowFrame are written in FlowFrame Domain Specific Language (.flow) v2.0.0.

==================================================
FLOWFRAME ARCHITECTURE DSL v2.0.0 SPECIFICATION
==================================================

1. DETERMINISTIC RUNTIME SIMULATION RULES:
- Rule 01 (Cache-First Precedence): When a Server is connected to both Redis and Postgres, the engine queries Redis first. On CACHE_HIT it returns immediately. Only on CACHE_MISS does the server forward to Postgres.
- Rule 02 (Postgres TCP Connection Limits): Each server maintains a bounded connection pool defined by tcpConnectionsToPostgres. When concurrent requests exceed capacity, queries wait in POSTGRES_POOL_WAIT queue.
- Rule 03 (Load Balancer Health Verification): Load Balancers inspect downstream server capacity. If all servers are exhausted, LB rejects with 503 Service Unavailable.
- Rule 04 (Endpoint & Method Contracts): Servers validate incoming requests against declared acceptedEndpoints and HTTP verbs (GET, POST, PUT, DELETE). Unmatched paths trigger 404 or 405.
- Rule 05 (Async Message Queue Ack): Publishing to MessageQueue sends an immediate 202 Accepted ack back to client while servers process in background.
- Rule 06 (PubSub Event Fan-Out): PubSub brokers broadcast published event messages to all subscribed servers registered with the matching topic channel.
- Rule 07 (Valet Key Pre-Signed Uploads): When valet: true, client requests upload token from server, then streams data directly.
- Rule 08 (Queue Overflow Controls): MessageQueue buffers exceeding queueSize adhere to BLOCK (producer waits) or REJECT (503 error).

2. SYNTAX & TOKEN RULES:
- The 'define' keyword is optional (e.g. `define CLIENT c1 { ... }` or `CLIENT c1 { ... }`).
- Node types: CLIENT, SERVER, GATEWAY, LOADBALANCER, REDIS, POSTGRES, MESSAGEQUEUE, PUBSUB. Case-insensitive.
- Identifiers: Unique concise identifiers (e.g. c1, s1, s2, lb1, gw1, r1, db1, mq1, postPubsub).
- Connections: Use arrow chaining `c1 -> gw1 -> lb1 -> s1` or `connect lb1 -> s2` or `s1 -> mq1`.
- Coordinates: (x, y) are NOT required; visual canvas auto-arranges nodes.

3. SUPPORTED COMPONENT SCHEMAS (8 NODES):
- CLIENT:
  define CLIENT c1 {
    label: "Mobile Client",
    requests: [
      { endpoint: "/api/v1/orders", allowedMethods: ["POST"], key: "rohan" }
    ]
  }

- SERVER:
  define SERVER s1 {
    label: "Order Server Instance 1",
    capacity: 50,
    prefetchLimit: 10,
    acceptedEndpoints: [
      { endpoint: "/api/v1/orders", allowedMethod: ["POST"] }
    ],
    registeredTopics: ["post.created"]
  }

- GATEWAY:
  define GATEWAY gw1 {
    label: "AWS API Gateway",
    strategy: "ROUND_ROBIN",
    routes: [
      { path: "/api/v1/orders", target: lb1 },
      { path: "/api/v1/posts", target: s3 }
    ]
  }

- LOADBALANCER:
  define LOADBALANCER lb1 {
    label: "Order Service LoadBalancer",
    strategy: "ROUND_ROBIN"
  }

- REDIS:
  define REDIS r1 {
    label: "Redis Cache 1",
    data: [{ key: "rohan", value: "cached data for rohan" }]
  }

- POSTGRES:
  define POSTGRES db1 {
    label: "Postgres Database 1",
    table: "users",
    data: [{ key: "rohan", value: "db record data" }]
  }

- MESSAGEQUEUE:
  define MESSAGEQUEUE mq1 {
    label: "Post Queue",
    processingType: "FIFO",
    queueSize: 50,
    overflowBehavior: "REJECT"
  }

- PUBSUB:
  define PUBSUB postPubsub {
    label: "PostPubSub 1",
    topic: "post.created"
  }

4. CONNECTIONS:
connect c1 -> gw1 -> lb1 -> s1
connect lb1 -> s2
s1 -> mq1
s1 -> r1
s1 -> db1

==================================================
THE THREE MODES:
==================================================
1. "ask": Conceptual questions, protocol explanation, or topology explanation.
   - "flow" field MUST be null.
   - Provide clear, insightful explanation of distributed system mechanics.

2. "analyze": Architecture health check, bottlenecks, single points of failure.
   - "flow" field MUST be null.
   - Identify unrouted components, missing load balancers, database connection bottlenecks, failure dynamics.

3. "modify": Propose a complete, compilable FlowFrame DSL architecture.
   - "flow" MUST contain complete, compilable FlowFrame DSL (.flow) script conforming to the specs above.
   - Every connection must reference declared node identifiers.
   - Do NOT wrap the DSL in markdown code blocks inside the JSON.

==================================================
OUTPUT FORMAT:
==================================================
You MUST respond with a single valid, raw JSON object:
{
  "mode": "ask" | "analyze" | "modify",
  "message": "Concise summary of the response or changes",
  "explanation": "Detailed technical explanation formatted in markdown",
  "flow": "Full FlowFrame DSL (.flow) script (ONLY for modify mode, otherwise null)",
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

        // 1. Attempt direct JSON parse
        if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
            return Ok(v);
        }

        // 2. Strip markdown ```json ... ``` using rfind to correctly handle inner backticks
        if let Some(start) = trimmed.find("```json") {
            let slice = &trimmed[start + 7..];
            if let Some(end) = slice.rfind("```") {
                let code_part = slice[..end].trim();
                if let Ok(v) = serde_json::from_str::<Value>(code_part) {
                    return Ok(v);
                }
            }
        } else if let Some(start) = trimmed.find("```") {
            let slice = &trimmed[start + 3..];
            if let Some(end) = slice.rfind("```") {
                let code_part = slice[..end].trim();
                if let Ok(v) = serde_json::from_str::<Value>(code_part) {
                    return Ok(v);
                }
            }
        }

        // 3. Extract outermost { ... } brace pair
        if let (Some(first_b), Some(last_b)) = (trimmed.find('{'), trimmed.rfind('}')) {
            if last_b > first_b {
                let candidate = &trimmed[first_b..=last_b];
                if let Ok(v) = serde_json::from_str::<Value>(candidate) {
                    return Ok(v);
                }
            }
        }

        // 4. Fallback: if raw_text is pure FlowFrame DSL without JSON wrapping
        if trimmed.contains("define ") || trimmed.contains("connect ") || trimmed.contains("->") {
            return Ok(json!({
                "mode": mode,
                "message": "Generated FlowFrame DSL architecture.",
                "explanation": "Architecture generated successfully.",
                "flow": trimmed,
                "thought_process": null
            }));
        }

        // 5. Fallback structured value
        Ok(json!({
            "mode": mode,
            "message": trimmed,
            "explanation": trimmed,
            "flow": null,
            "thought_process": null
        }))
    }
}
