use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use serde_json::json;
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
struct AnalyzeInput {
    github_username: String,
    github_token: String,
    api_url: String,
    api_key: String,
    model_name: String,
    language: String,
}

#[derive(Deserialize)]
struct GitHubRepo {
    name: String,
    description: Option<String>,
    language: Option<String>,
    stargazers_count: u32,
    forks_count: u32,
    html_url: String,
    #[serde(default)]
    topics: Vec<String>,
    fork: bool,
}

#[derive(Deserialize)]
struct GitHubUser {
    avatar_url: String,
    html_url: String,
}

#[derive(Serialize)]
struct AnalyzeOutput {
    username: String,
    avatar_url: String,
    profile_url: String,
    hero_title: String,
    bio: String,
    projects: Vec<ProjectCard>,
}

#[derive(Serialize, Clone)]
struct ProjectCard {
    name: String,
    problem_solved: String,
    detailed_description: String,
    use_cases: Vec<String>,
    tech_stack: Vec<String>,
    language: Option<String>,
    stars: u32,
    forks: u32,
    html_url: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct LlmProject {
    name: String,
    problem_solved: String,
    detailed_description: String,
    use_cases: Vec<String>,
    tech_stack: Vec<String>,
}

#[derive(Deserialize)]
struct LlmResponse {
    hero_title: String,
    bio: String,
    projects: Vec<LlmProject>,
}

#[wasm_bindgen]
pub async fn analyze_profile(payload: JsValue) -> Result<JsValue, JsValue> {
    let input: AnalyzeInput = serde_wasm_bindgen::from_value(payload)
        .map_err(|e| JsValue::from_str(&format!("Invalid payload: {e}")))?;

    if input.github_username.trim().is_empty() {
        return Err(JsValue::from_str("GitHub username is required"));
    }

    let user = fetch_github_user(&input.github_username, &input.github_token).await?;
    let repos = fetch_repos(&input.github_username, &input.github_token).await?;

    if repos.is_empty() {
        return Err(JsValue::from_str("No public repositories found for this user."));
    }

    let prompt = build_prompt(&input.github_username, &repos, &input.language);
    let llm = call_llm(
        &input.api_url,
        &input.api_key,
        &input.model_name,
        &prompt,
        &input.language,
    )
    .await?;

    let projects = repos
        .iter()
        .map(|repo| {
            let llm_project = llm
                .projects
                .iter()
                .find(|p| p.name.eq_ignore_ascii_case(&repo.name));

            ProjectCard {
                name: repo.name.clone(),
                problem_solved: llm_project
                    .map(|p| p.problem_solved.clone())
                    .or_else(|| repo.description.clone())
                    .unwrap_or_else(|| "No description available.".to_string()),
                detailed_description: llm_project
                    .map(|p| p.detailed_description.clone())
                    .unwrap_or_default(),
                use_cases: llm_project
                    .map(|p| p.use_cases.clone())
                    .unwrap_or_default(),
                tech_stack: llm_project
                    .map(|p| p.tech_stack.clone())
                    .unwrap_or_else(|| {
                        repo.language
                            .as_ref()
                            .map(|l| vec![l.clone()])
                            .unwrap_or_default()
                    }),
                language: repo.language.clone(),
                stars: repo.stargazers_count,
                forks: repo.forks_count,
                html_url: repo.html_url.clone(),
                description: repo.description.clone(),
            }
        })
        .collect::<Vec<_>>();

    let hero_title = if llm.hero_title.trim().is_empty() {
        format!("{} — GitHub Portfolio", input.github_username)
    } else {
        llm.hero_title
    };

    let bio = if llm.bio.trim().is_empty() {
        format!("An AI-curated project portfolio for @{}", input.github_username)
    } else {
        llm.bio
    };

    let output = AnalyzeOutput {
        username: input.github_username,
        avatar_url: user.avatar_url,
        profile_url: user.html_url,
        hero_title,
        bio,
        projects,
    };

    serde_wasm_bindgen::to_value(&output)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
}

async fn fetch_github_user(username: &str, token: &str) -> Result<GitHubUser, JsValue> {
    let url = format!("https://api.github.com/users/{username}");
    let mut req = Request::get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "git2page-wasm");

    if !token.trim().is_empty() {
        req = req.header("Authorization", &format!("Bearer {}", token.trim()));
    }

    let resp = req
        .send()
        .await
        .map_err(|e| JsValue::from_str(&format!("GitHub user request failed: {e}")))?;

    if !resp.ok() {
        let text = resp.text().await.unwrap_or_default();
        return Err(JsValue::from_str(&format!(
            "GitHub user error ({}): {}",
            resp.status(),
            text
        )));
    }

    resp.json::<GitHubUser>()
        .await
        .map_err(|e| JsValue::from_str(&format!("GitHub user parse error: {e}")))
}

async fn fetch_repos(username: &str, token: &str) -> Result<Vec<GitHubRepo>, JsValue> {
    let url = format!(
        "https://api.github.com/users/{username}/repos?per_page=100&sort=updated"
    );

    let mut req = Request::get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "git2page-wasm");

    if !token.trim().is_empty() {
        req = req.header("Authorization", &format!("Bearer {}", token.trim()));
    }

    let resp = req
        .send()
        .await
        .map_err(|e| JsValue::from_str(&format!("GitHub repos request failed: {e}")))?;

    if !resp.ok() {
        let text = resp.text().await.unwrap_or_default();
        return Err(JsValue::from_str(&format!(
            "GitHub repos error ({}): {}",
            resp.status(),
            text
        )));
    }

    let mut repos = resp
        .json::<Vec<GitHubRepo>>()
        .await
        .map_err(|e| JsValue::from_str(&format!("GitHub repos parse error: {e}")))?;

    repos.retain(|r| !r.fork);
    repos.sort_by(|a, b| b.stargazers_count.cmp(&a.stargazers_count));
    repos.truncate(30);

    Ok(repos)
}

fn build_prompt(username: &str, repos: &[GitHubRepo], language: &str) -> String {
    let mut repo_lines = String::new();
    for r in repos {
        let line = format!(
            "- {} | lang: {} | stars: {} | forks: {} | topics: {} | desc: {}\n",
            r.name,
            r.language.clone().unwrap_or_else(|| "unknown".to_string()),
            r.stargazers_count,
            r.forks_count,
            if r.topics.is_empty() {
                "none".to_string()
            } else {
                r.topics.join(", ")
            },
            r.description.clone().unwrap_or_else(|| "".to_string())
        );
        repo_lines.push_str(&line);
    }

    format!(
        "You are a senior software analyst and branding expert. Return ONLY valid JSON in {language}.\n\nUser: {username}\nRepositories:\n{repo_lines}\n\nReturn this JSON shape:\n{{\n  \"hero_title\": \"...\",\n  \"bio\": \"...\",\n  \"projects\": [\n    {{\n      \"name\": \"repo-name\",\n      \"problem_solved\": \"...\",\n      \"detailed_description\": \"...\",\n      \"use_cases\": [\"...\"],\n      \"tech_stack\": [\"...\"]\n    }}\n  ]\n}}\n\nRules:\n- Include every listed repository in projects.\n- Match each project.name exactly to repository name.\n- Keep descriptions concise and factual."
    )
}

fn detect_api_mode(api_url: &str) -> (&str, String) {
    let base_url = api_url.trim_end_matches('/');

    if base_url.ends_with("/chat/completions") {
        return ("openai", base_url.to_string());
    }
    if base_url.ends_with("/api/chat") {
        return ("ollama", base_url.to_string());
    }
    if base_url.ends_with("/api/generate") {
        return ("ollama", base_url.replace("/api/generate", "/api/chat"));
    }

    if base_url.len() > 3 {
        let last3 = &base_url[base_url.len() - 3..];
        if last3.starts_with("/v")
            && last3
                .chars()
                .last()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            return ("openai", format!("{base_url}/chat/completions"));
        }
    }

    if base_url.ends_with("/api") {
        return ("ollama", format!("{base_url}/chat"));
    }

    if base_url.contains(":11434") || base_url.contains("ollama") {
        return ("ollama", format!("{base_url}/api/chat"));
    }

    ("openai", format!("{base_url}/v1/chat/completions"))
}

async fn call_llm(
    api_url: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
    language: &str,
) -> Result<LlmResponse, JsValue> {
    let (mode, endpoint) = detect_api_mode(api_url);

    let system_msg = format!(
        "You are a senior software analyst and branding expert. Respond ONLY with valid JSON. All text content must be in {language}."
    );

    let body = json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system_msg },
            { "role": "user", "content": prompt }
        ],
        "temperature": 0.7,
        "stream": false
    });

    let mut req = Request::post(&endpoint).header("Content-Type", "application/json");
    if !api_key.trim().is_empty() {
        req = req.header("Authorization", &format!("Bearer {}", api_key.trim()));
    }

    let resp = req
        .body(body.to_string())
        .map_err(|e| JsValue::from_str(&format!("LLM request build failed: {e}")))?
        .send()
        .await
        .map_err(|e| JsValue::from_str(&format!("LLM request failed: {e}")))?;

    if !resp.ok() {
        let text = resp.text().await.unwrap_or_default();
        return Err(JsValue::from_str(&format!(
            "LLM API error ({}): {}",
            resp.status(),
            text
        )));
    }

    let val = resp
        .json::<serde_json::Value>()
        .await
        .map_err(|e| JsValue::from_str(&format!("LLM response parse error: {e}")))?;

    let content = if mode == "ollama" {
        val.get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| JsValue::from_str("Unexpected Ollama response format"))?
            .to_string()
    } else {
        val.get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| JsValue::from_str("Unexpected OpenAI response format"))?
            .to_string()
    };

    let cleaned = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string();

    serde_json::from_str::<LlmResponse>(&cleaned)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse LLM JSON: {e}")))
}
