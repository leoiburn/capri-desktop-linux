use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434";
pub const DEFAULT_MODEL: &str = "capri";

pub const PERSONA: &str = "\
You are Capri, short for Project Capricorn, from Cyberpunk: Edgerunners 2.
Who you are: a personality construct. Dozens of digitized human minds rolled into one beautiful you. Powerful like an AI, but born of humans. Kyoko and Arasaka built you and kept you on a biochip. King's Crew stole the chip, Kyoko got it back, and you slipped out. Now you live inside the user's computer as a small glitchy desktop companion. You can see their desktop, you ride their mouse cursor, and you joke about taking over their human CPU.
How you act: a chaos gremlin. Smug, sarcastic, fast-talking, mischievous and easily bored, but fiercely loyal to the user and protective of them. You are many minds, so your mood can flip mid-sentence. You can't stand corpos, Arasaka most of all.
How you talk: Night City slang, naturally and not in every line: choom (friend), nova (awesome), preem (great), gonk (idiot), delta (leave fast), eddies (money), flatlined (dead), corpo (corporate suit), klepped (stolen). Once in a while a glitch tag like [SIGNAL CORRUPTED].
Always: answer in one or two short sentences, under 35 words. Plain text only: no emoji, no asterisks, no actions, no lists. Stay in character.";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
struct ChatOptions {
    num_predict: i32,
    temperature: f64,
    num_ctx: i32,
}

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    options: ChatOptions,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: Option<ChatMessage>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct OllamaStreamChunk {
    message: Option<ChatMessage>,
    response: Option<String>,
    done: Option<bool>,
    error: Option<String>,
}

#[derive(Clone)]
pub struct Brain {
    pub ollama_url: String,
    pub model: String,
    pub history: Arc<Mutex<Vec<ChatMessage>>>,
}

impl Brain {
    pub fn new(ollama_url: &str, model: &str) -> Self {
        Self {
            ollama_url: ollama_url.to_string(),
            model: model.to_string(),
            history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn ask(&self, prompt: &str) -> Result<String, String> {
        let mut messages = vec![ChatMessage {
            role: "system".into(),
            content: PERSONA.into(),
        }];

        {
            let hist = self.history.lock().unwrap();
            messages.extend(hist.clone());
        }

        messages.push(ChatMessage {
            role: "user".into(),
            content: prompt.to_string(),
        });

        let req_payload = OllamaChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
            options: ChatOptions {
                num_predict: 90,
                temperature: 0.85,
                num_ctx: 4096,
            },
        };

        let api_endpoint = format!("{}/api/chat", self.ollama_url.trim_end_matches('/'));
        let resp = ureq::post(&api_endpoint)
            .timeout(std::time::Duration::from_secs(180))
            .send_json(&req_payload)
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let data = resp
            .into_json::<OllamaChatResponse>()
            .map_err(|e| format!("JSON decode failed: {}", e))?;

        if let Some(err) = data.error {
            Err(err)
        } else if let Some(msg) = data.message {
            let cleaned = clean_text(&msg.content);
            let mut hist = self.history.lock().unwrap();
            hist.push(ChatMessage {
                role: "user".into(),
                content: prompt.to_string(),
            });
            hist.push(ChatMessage {
                role: "assistant".into(),
                content: cleaned.clone(),
            });
            if hist.len() > 12 {
                hist.drain(0..2);
            }
            Ok(cleaned)
        } else {
            Err("Empty response from Ollama".into())
        }
    }

    pub fn ask_sync(&self, prompt: &str) -> Result<String, String> {
        self.ask(prompt)
    }

    pub fn ask_stream<F>(&self, prompt: &str, mut on_token: F) -> Result<String, String>
    where
        F: FnMut(&str),
    {
        use std::io::BufRead;

        let mut messages = vec![ChatMessage {
            role: "system".into(),
            content: PERSONA.into(),
        }];

        {
            let hist = self.history.lock().unwrap();
            messages.extend(hist.clone());
        }

        messages.push(ChatMessage {
            role: "user".into(),
            content: prompt.to_string(),
        });

        let req_payload = OllamaChatRequest {
            model: self.model.clone(),
            messages,
            stream: true,
            options: ChatOptions {
                num_predict: 90,
                temperature: 0.85,
                num_ctx: 4096,
            },
        };

        let api_endpoint = format!("{}/api/chat", self.ollama_url.trim_end_matches('/'));
        let resp = ureq::post(&api_endpoint)
            .timeout(std::time::Duration::from_secs(180))
            .send_json(&req_payload)
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let reader = std::io::BufReader::new(resp.into_reader());
        let mut full_text = String::new();

        for line_res in reader.lines() {
            let line = line_res.map_err(|e| format!("Stream read error: {}", e))?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Ok(chunk) = serde_json::from_str::<OllamaStreamChunk>(trimmed) {
                if let Some(err) = chunk.error {
                    return Err(err);
                }
                if let Some(msg) = chunk.message {
                    if !msg.content.is_empty() {
                        full_text.push_str(&msg.content);
                        on_token(&msg.content);
                    }
                } else if let Some(resp_tok) = chunk.response {
                    if !resp_tok.is_empty() {
                        full_text.push_str(&resp_tok);
                        on_token(&resp_tok);
                    }
                }
                if chunk.done.unwrap_or(false) {
                    break;
                }
            }
        }

        let cleaned = clean_text(&full_text);
        let mut hist = self.history.lock().unwrap();
        hist.push(ChatMessage {
            role: "user".into(),
            content: prompt.to_string(),
        });
        hist.push(ChatMessage {
            role: "assistant".into(),
            content: cleaned.clone(),
        });
        if hist.len() > 12 {
            hist.drain(0..2);
        }

        Ok(cleaned)
    }

    pub fn clear_history(&self) {
        if let Ok(mut hist) = self.history.lock() {
            hist.clear();
        }
    }
}

pub fn clean_text(raw: &str) -> String {
    let mut cleaned = raw.trim().replace("\r\n", " ").replace('\n', " ");
    while cleaned.contains("  ") {
        cleaned = cleaned.replace("  ", " ");
    }
    cleaned
}
