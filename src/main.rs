use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer};
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

// ─── Request / Response Types ───────────────────────────────────────────────

#[derive(Deserialize)]
struct AnalyzeRequest {
    github_username: String,
    api_url: String,
    api_key: String,
    model_name: String,
    #[serde(default)]
    github_token: String,
    #[serde(default = "default_language")]
    language: String,
}

fn default_language() -> String {
    "Turkish".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct RepoInfo {
    name: String,
    description: Option<String>,
    language: Option<String>,
    stars: u32,
    forks: u32,
    html_url: String,
    topics: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LlmProject {
    name: String,
    problem_solved: String,
    detailed_description: String,
    use_cases: Vec<String>,
    tech_stack: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LlmResponse {
    hero_title: String,
    bio: String,
    projects: Vec<LlmProject>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LlmBatchResponse {
    projects: Vec<LlmProject>,
}

#[derive(Serialize)]
struct AnalyzeResponse {
    username: String,
    avatar_url: String,
    profile_url: String,
    hero_title: String,
    bio: String,
    projects: Vec<ProjectCard>,
}

#[derive(Serialize)]
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

// ─── GitHub API Types ───────────────────────────────────────────────────────

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

#[derive(Deserialize)]
struct GitHubContent {
    content: Option<String>,
    encoding: Option<String>,
}

// ─── GitHub Module ──────────────────────────────────────────────────────────

async fn fetch_github_user(client: &Client, username: &str, token: &str) -> Result<GitHubUser> {
    let url = format!("https://api.github.com/users/{}", username);
    let mut req = client
        .get(&url)
        .header("User-Agent", "git2page-rust")
        .header("Accept", "application/vnd.github.v3+json");
    if !token.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", token));
    }
    let resp = req.send().await?;

    if !resp.status().is_success() {
        anyhow::bail!("GitHub user not found: {}", resp.status());
    }

    let user: GitHubUser = resp.json().await?;
    Ok(user)
}

async fn fetch_repos(client: &Client, username: &str, token: &str) -> Result<Vec<RepoInfo>> {
    let url = format!(
        "https://api.github.com/users/{}/repos?sort=stars&per_page=30&type=owner",
        username
    );
    let mut req = client
        .get(&url)
        .header("User-Agent", "git2page-rust")
        .header("Accept", "application/vnd.github.mercy-preview+json");
    if !token.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", token));
    }
    let resp = req.send().await?;

    if !resp.status().is_success() {
        anyhow::bail!("Failed to fetch repos: {}", resp.status());
    }

    let gh_repos: Vec<GitHubRepo> = resp.json().await?;

    let repos: Vec<RepoInfo> = gh_repos
        .into_iter()
        .filter(|r| !r.fork)
        .map(|r| RepoInfo {
            name: r.name,
            description: r.description,
            language: r.language,
            stars: r.stargazers_count,
            forks: r.forks_count,
            html_url: r.html_url,
            topics: r.topics,
        })
        .collect();

    Ok(repos)
}

async fn fetch_file_content(
    client: &Client,
    username: &str,
    repo: &str,
    path: &str,
    token: &str,
) -> Result<String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}",
        username, repo, path
    );
    let mut req = client
        .get(&url)
        .header("User-Agent", "git2page-rust")
        .header("Accept", "application/vnd.github.v3+json");
    if !token.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", token));
    }
    let resp = req.send().await?;

    if !resp.status().is_success() {
        anyhow::bail!("File not found: {} in {}/{}", path, username, repo);
    }

    let content: GitHubContent = resp.json().await?;
    match (content.content, content.encoding) {
        (Some(encoded), Some(enc)) if enc == "base64" => {
            let cleaned: String = encoded.chars().filter(|c| !c.is_whitespace()).collect();
            let decoded = base64_decode(&cleaned)?;
            Ok(decoded)
        }
        _ => anyhow::bail!("Unexpected encoding for {}/{}/{}", username, repo, path),
    }
}

async fn fetch_repo_root_files(
    client: &Client,
    username: &str,
    repo: &str,
    token: &str,
) -> Result<Vec<String>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/",
        username, repo
    );
    let mut req = client
        .get(&url)
        .header("User-Agent", "git2page-rust")
        .header("Accept", "application/vnd.github.v3+json");
    if !token.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", token));
    }
    let resp = req.send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("Failed to list repo contents: {}", resp.status());
    }
    let items: Vec<serde_json::Value> = resp.json().await?;
    let files: Vec<String> = items
        .iter()
        .filter(|item| item["type"].as_str() == Some("file"))
        .filter_map(|item| item["name"].as_str().map(|s| s.to_string()))
        .collect();
    Ok(files)
}

async fn fetch_src_dir_files(
    client: &Client,
    username: &str,
    repo: &str,
    token: &str,
) -> Result<Vec<String>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/src",
        username, repo
    );
    let mut req = client
        .get(&url)
        .header("User-Agent", "git2page-rust")
        .header("Accept", "application/vnd.github.v3+json");
    if !token.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", token));
    }
    let resp = req.send().await?;
    if !resp.status().is_success() {
        return Ok(Vec::new());
    }
    let items: Vec<serde_json::Value> = resp.json().await?;
    let files: Vec<String> = items
        .iter()
        .filter(|item| item["type"].as_str() == Some("file"))
        .filter_map(|item| item["name"].as_str().map(|s| format!("src/{}", s)))
        .collect();
    Ok(files)
}

fn is_source_file(name: &str) -> bool {
    let ext_list = [
        ".py", ".js", ".ts", ".rs", ".go", ".java", ".rb", ".php",
        ".cs", ".swift", ".kt", ".dart", ".c", ".cpp", ".h", ".vue",
        ".svelte", ".jsx", ".tsx", ".lua", ".sh", ".pl",
    ];
    let lower = name.to_lowercase();
    ext_list.iter().any(|ext| lower.ends_with(ext))
}

fn is_main_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    let main_names = [
        "main.", "app.", "index.", "server.", "program.", "__main__.",
        "mod.", "lib.", "init.", "cli.", "run.", "start.", "bot.",
    ];
    main_names.iter().any(|m| lower.contains(m))
}

fn base64_decode(input: &str) -> Result<String> {
    // Simple base64 decoder
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut buf: Vec<u8> = Vec::new();
    let mut bits: u32 = 0;
    let mut bit_count: u32 = 0;

    for &byte in input.as_bytes() {
        if byte == b'=' {
            break;
        }
        let val = match TABLE.iter().position(|&b| b == byte) {
            Some(v) => v as u32,
            None => continue,
        };
        bits = (bits << 6) | val;
        bit_count += 6;
        if bit_count >= 8 {
            bit_count -= 8;
            buf.push((bits >> bit_count) as u8);
            bits &= (1 << bit_count) - 1;
        }
    }

    String::from_utf8(buf).map_err(|e| anyhow::anyhow!("UTF-8 decode error: {}", e))
}

// ─── Analysis Module ────────────────────────────────────────────────────────

async fn gather_repo_context(
    client: &Client,
    username: &str,
    repos: &[RepoInfo],
    token: &str,
) -> Vec<String> {
    let mut contexts = Vec::new();
    let repo_count = repos.len();
    let max_readme_chars: usize = if repo_count > 15 { 600 } else { 1000 };
    let max_source_chars: usize = if repo_count > 15 { 800 } else { 1200 };
    let max_manifest_chars: usize = 300;

    for (i, repo) in repos.iter().enumerate() {
        eprintln!("[context] ({}/{}) Analyzing repo: {}", i + 1, repo_count, repo.name);

        let mut ctx = format!(
            "Repo: {} | Stars: {} | Forks: {} | Language: {} | Description: {}",
            repo.name,
            repo.stars,
            repo.forks,
            repo.language.as_deref().unwrap_or("N/A"),
            repo.description.as_deref().unwrap_or("N/A")
        );

        if !repo.topics.is_empty() {
            ctx.push_str(&format!(" | Topics: {}", repo.topics.join(", ")));
        }

        let mut has_readme = false;
        // Try README first (case-insensitive: try both)
        for readme_name in &["README.md", "readme.md", "Readme.md"] {
            if let Ok(readme) = fetch_file_content(client, username, &repo.name, readme_name, token).await {
                let truncated: String = readme.chars().take(max_readme_chars).collect();
                ctx.push_str(&format!("\nREADME (truncated):\n{}", truncated));
                has_readme = true;
                break;
            }
        }

        // Try manifest files for tech stack info
        for manifest in &["Cargo.toml", "package.json", "pyproject.toml", "go.mod", "requirements.txt", "setup.py", "build.gradle", "pom.xml"] {
            if let Ok(content) =
                fetch_file_content(client, username, &repo.name, manifest, token).await
            {
                let truncated: String = content.chars().take(max_manifest_chars).collect();
                ctx.push_str(&format!("\n{} (truncated):\n{}", manifest, truncated));
                break;
            }
        }

        // If no README, dynamically discover and fetch source files
        if !has_readme {
            let mut found_source = false;

            // List root directory files
            let mut all_files: Vec<String> = Vec::new();
            if let Ok(root_files) = fetch_repo_root_files(client, username, &repo.name, token).await {
                all_files.extend(root_files);
            }
            // Also list src/ directory
            if let Ok(src_files) = fetch_src_dir_files(client, username, &repo.name, token).await {
                all_files.extend(src_files);
            }

            if !all_files.is_empty() {
                // Log discovered files
                let file_list: String = all_files.iter().take(20).cloned().collect::<Vec<_>>().join(", ");
                ctx.push_str(&format!("\nFILE STRUCTURE: [{}]", file_list));

                // Priority 1: main source files (main.py, index.js, app.py, etc.)
                let main_sources: Vec<&String> = all_files.iter()
                    .filter(|f| is_source_file(f) && is_main_file(f))
                    .collect();

                // Priority 2: any source files
                let any_sources: Vec<&String> = all_files.iter()
                    .filter(|f| is_source_file(f))
                    .collect();

                let target_files = if !main_sources.is_empty() { main_sources } else { any_sources };

                // Fetch up to 2 source files
                let mut files_fetched = 0;
                for file_path in target_files.iter().take(2) {
                    if let Ok(content) = fetch_file_content(client, username, &repo.name, file_path, token).await {
                        let truncated: String = content.chars().take(max_source_chars).collect();
                        ctx.push_str(&format!("\nSOURCE CODE ({}):\n{}", file_path, truncated));
                        found_source = true;
                        files_fetched += 1;
                    }
                }
                eprintln!("[context]   → {} files discovered, {} source files fetched", all_files.len(), files_fetched);
            }

            if !found_source {
                ctx.push_str("\n[No README or source files found — analyze from repo name, language, and description]");
                eprintln!("[context]   → No source files found, metadata only");
            }
        }

        contexts.push(ctx);
    }

    contexts
}

fn build_llm_prompt_full(username: &str, contexts: &[String], language: &str, repo_names: &[String]) -> String {
    let repo_data = contexts.join("\n\n---\n\n");
    let names_list = repo_names.join(", ");

    format!(
        r#"You are a senior software analyst and branding expert. Analyze the following GitHub profile data deeply.

CRITICAL RULES:
- Respond ENTIRELY in {lang}.
- You MUST generate an entry for EVERY repository listed below. Do NOT skip any.
- Required repos (you MUST include ALL of these): [{names}]
- If a project has SOURCE CODE provided, READ and UNDERSTAND the code to determine what the project does.
- If a project has NO README, use the code, dependencies, description, language, and metadata to infer the project's purpose. NEVER leave a project without analysis.
- If a project only has metadata (name, language, description), use that to intelligently infer what the project does and generate a meaningful description.
- Be specific and technical in your descriptions — do NOT use generic phrases like "this is a project".
- Every project MUST have a detailed_description (3-5 sentences) and at least 2 use_cases.
- Respond ONLY with valid JSON. No markdown fences, no extra text.

GitHub User: {user}

Repository Data:
{repos}

Respond in this exact JSON format (include ALL {count} repositories):
{{
  "hero_title": "A short, impactful professional title for this developer (in {lang})",
  "bio": "A 3-4 sentence professional biography highlighting their expertise, tech focus, and impact (in {lang})",
  "projects": [
    {{
      "name": "exact-repo-name",
      "problem_solved": "One clear sentence about the core problem this project solves (in {lang})",
      "detailed_description": "3-5 sentence deep technical description of what the project does, its architecture, and key features (in {lang})",
      "use_cases": ["Specific use case 1 (in {lang})", "Specific use case 2 (in {lang})", "Specific use case 3 (in {lang})"],
      "tech_stack": ["technology1", "technology2", "technology3"]
    }}
  ]
}}"#,
        lang = language,
        user = username,
        repos = repo_data,
        names = names_list,
        count = repo_names.len(),
    )
}

fn build_llm_prompt_batch(contexts: &[String], language: &str, repo_names: &[String]) -> String {
    let repo_data = contexts.join("\n\n---\n\n");
    let names_list = repo_names.join(", ");

    format!(
        r#"You are a senior software analyst. Analyze the following repositories deeply.

CRITICAL RULES:
- Respond ENTIRELY in {lang}.
- You MUST generate an entry for EVERY repository: [{names}]
- If a project has SOURCE CODE, READ and UNDERSTAND the code to determine what it does.
- If a project has NO README, use code, dependencies, description, language, and metadata to infer purpose.
- Be specific and technical. Do NOT use generic phrases.
- Every project MUST have detailed_description (3-5 sentences) and at least 2 use_cases.
- Respond ONLY with valid JSON. No markdown fences, no extra text.

Repository Data:
{repos}

Respond in this exact JSON format (include ALL {count} repositories):
{{
  "projects": [
    {{
      "name": "exact-repo-name",
      "problem_solved": "One clear sentence (in {lang})",
      "detailed_description": "3-5 sentence technical description (in {lang})",
      "use_cases": ["Use case 1 (in {lang})", "Use case 2 (in {lang})"],
      "tech_stack": ["tech1", "tech2"]
    }}
  ]
}}"#,
        lang = language,
        repos = repo_data,
        names = names_list,
        count = repo_names.len(),
    )
}

// ─── LLM Client ─────────────────────────────────────────────────────────────

fn detect_api_mode(api_url: &str) -> (&str, String) {
    let base_url = api_url.trim_end_matches('/');

    // If user already provided a full endpoint path, use it as-is
    if base_url.ends_with("/chat/completions") {
        return ("openai", base_url.to_string());
    }
    if base_url.ends_with("/api/chat") {
        return ("ollama", base_url.to_string());
    }
    if base_url.ends_with("/api/generate") {
        return ("ollama", base_url.replace("/api/generate", "/api/chat"));
    }

    // If URL ends with /v1, /v2, /v3, /v4 etc → OpenAI-compatible mode
    if base_url.len() > 3 {
        let last3 = &base_url[base_url.len()-3..];
        if last3.starts_with("/v") && last3.chars().last().map_or(false, |c| c.is_ascii_digit()) {
            return ("openai", format!("{}/chat/completions", base_url));
        }
    }

    // If URL ends with /api → Ollama native
    if base_url.ends_with("/api") {
        return ("ollama", format!("{}/chat", base_url));
    }

    // Auto-detect: if URL contains common Ollama ports or paths, use Ollama native
    if base_url.contains(":11434") || base_url.contains("ollama") {
        return ("ollama", format!("{}/api/chat", base_url));
    }

    // Default: try OpenAI-compatible
    ("openai", format!("{}/v1/chat/completions", base_url))
}

async fn call_llm(
    client: &Client,
    api_url: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
    language: &str,
) -> Result<LlmResponse> {
    let (mode, endpoint) = detect_api_mode(api_url);

    let system_msg = format!(
        "You are a senior software analyst and branding expert. Respond ONLY with valid JSON. No markdown fences, no extra text. All text content must be in {}.",
        language
    );

    let body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": system_msg
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "temperature": 0.7,
        "stream": false
    });

    let mut req = client
        .post(&endpoint)
        .header("Content-Type", "application/json");

    if !api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", api_key));
    }

    eprintln!("[call_llm] Sending request to: {}", endpoint);
    eprintln!("[call_llm] Body size: {} bytes", body.to_string().len());
    let resp = match req.json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[call_llm] Request error: {:?}", e);
            return Err(anyhow::anyhow!("error sending request for url ({}): {}", endpoint, e));
        }
    };

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("LLM API error ({}): {}", status, text);
    }

    let resp_json: serde_json::Value = resp.json().await?;

    // Extract content based on API mode
    // Ollama native: { "message": { "content": "..." } }
    // OpenAI compat: { "choices": [{ "message": { "content": "..." } }] }
    let content = if mode == "ollama" {
        resp_json["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Unexpected Ollama response format: {}", resp_json))?
    } else {
        resp_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Unexpected OpenAI response format: {}", resp_json))?
    };

    // Parse JSON from the content (strip markdown code fences if present)
    let cleaned = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let llm_resp: LlmResponse = serde_json::from_str(cleaned)
        .map_err(|e| anyhow::anyhow!("Failed to parse LLM JSON: {}. Raw: {}", e, cleaned))?;

    Ok(llm_resp)
}

async fn call_llm_batch(
    client: &Client,
    api_url: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
    language: &str,
) -> Result<LlmBatchResponse> {
    let (mode, endpoint) = detect_api_mode(api_url);

    let system_msg = format!(
        "You are a senior software analyst. Respond ONLY with valid JSON. No markdown fences, no extra text. All text content must be in {}.",
        language
    );

    let body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": system_msg
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "temperature": 0.7,
        "stream": false
    });

    let mut req = client
        .post(&endpoint)
        .header("Content-Type", "application/json");

    if !api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", api_key));
    }

    eprintln!("[call_llm_batch] Sending request to: {}", endpoint);
    eprintln!("[call_llm_batch] Body size: {} bytes", body.to_string().len());
    let resp = match req.json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[call_llm_batch] Request error: {:?}", e);
            return Err(anyhow::anyhow!("error sending request for url ({}): {}", endpoint, e));
        }
    };

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("LLM API error ({}): {}", status, text);
    }

    let resp_json: serde_json::Value = resp.json().await?;

    let content = if mode == "ollama" {
        resp_json["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Unexpected Ollama response format: {}", resp_json))?
    } else {
        resp_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Unexpected OpenAI response format: {}", resp_json))?
    };

    let cleaned = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let batch_resp: LlmBatchResponse = serde_json::from_str(cleaned)
        .map_err(|e| anyhow::anyhow!("Failed to parse batch LLM JSON: {}. Raw: {}", e, cleaned))?;

    Ok(batch_resp)
}

// ─── Config Endpoint ────────────────────────────────────────────────────────

async fn get_config() -> HttpResponse {
    let api_url = std::env::var("LLM_API_URL")
        .unwrap_or_else(|_| "https://ollama.com".to_string());
    let model = std::env::var("LLM_MODEL")
        .unwrap_or_else(|_| "llama3".to_string());
    let has_github_token = !std::env::var("GITHUB_TOKEN").unwrap_or_default().is_empty();
    let has_api_key = !std::env::var("LLM_API_KEY").unwrap_or_default().is_empty();

    HttpResponse::Ok().json(serde_json::json!({
        "api_url": api_url,
        "model": model,
        "has_github_token": has_github_token,
        "has_api_key": has_api_key
    }))
}

fn env_or(form_val: &str, env_key: &str) -> String {
    if form_val.is_empty() {
        let default = match env_key {
            "LLM_API_URL" => "https://ollama.com",
            "LLM_MODEL" => "llama3",
            _ => "",
        };
        std::env::var(env_key).unwrap_or_else(|_| default.to_string())
    } else {
        form_val.to_string()
    }
}

// ─── Analyze Endpoint ───────────────────────────────────────────────────────

async fn analyze(body: web::Json<AnalyzeRequest>) -> HttpResponse {
    let github_token = env_or(&body.github_token, "GITHUB_TOKEN");
    let api_url = env_or(&body.api_url, "LLM_API_URL");
    let api_key = env_or(&body.api_key, "LLM_API_KEY");
    let model_name = env_or(&body.model_name, "LLM_MODEL");
    let language = if body.language.is_empty() { "Turkish".to_string() } else { body.language.clone() };

    eprintln!("[analyze] Request received for user: {}", body.github_username);
    eprintln!("[analyze] API URL: {}, Model: {}, Language: {}", api_url, model_name, language);
    eprintln!("[analyze] GitHub token: {}", if github_token.is_empty() { "not set" } else { "set (from env or form)" });

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .unwrap_or_default();

    // 1. Fetch GitHub user info
    eprintln!("[analyze] Step 1: Fetching GitHub user info...");
    let user = match fetch_github_user(&client, &body.github_username, &github_token).await {
        Ok(u) => {
            eprintln!("[analyze] GitHub user fetched OK");
            u
        }
        Err(e) => {
            eprintln!("[analyze] ERROR - GitHub user: {}", e);
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("GitHub user error: {}", e)
            }));
        }
    };

    // 2. Fetch repos
    eprintln!("[analyze] Step 2: Fetching repos...");
    let repos = match fetch_repos(&client, &body.github_username, &github_token).await {
        Ok(r) => {
            eprintln!("[analyze] Fetched {} repos", r.len());
            r
        }
        Err(e) => {
            eprintln!("[analyze] ERROR - Repos: {}", e);
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("GitHub repos error: {}", e)
            }));
        }
    };

    if repos.is_empty() {
        eprintln!("[analyze] ERROR - No repos found");
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No public repositories found for this user."
        }));
    }

    // 3. Gather context from repos
    eprintln!("[analyze] Step 3: Gathering repo context...");
    let contexts = gather_repo_context(&client, &body.github_username, &repos, &github_token).await;
    eprintln!("[analyze] Gathered context for {} repos", contexts.len());

    // 4. Batch LLM calls (max ~8 repos per batch to avoid timeout)
    let batch_size = 8;
    let (mode, endpoint) = detect_api_mode(&api_url);
    eprintln!("[analyze] Step 4: Calling LLM in batches (mode={}, endpoint={})", mode, endpoint);

    let mut all_llm_projects: Vec<LlmProject> = Vec::new();
    let mut hero_title = String::new();
    let mut bio = String::new();

    let total_batches = (contexts.len() + batch_size - 1) / batch_size;

    for (batch_idx, chunk_start) in (0..contexts.len()).step_by(batch_size).enumerate() {
        let chunk_end = std::cmp::min(chunk_start + batch_size, contexts.len());
        let batch_contexts = &contexts[chunk_start..chunk_end];
        let batch_names: Vec<String> = repos[chunk_start..chunk_end]
            .iter()
            .map(|r| r.name.clone())
            .collect();

        eprintln!(
            "[analyze] Batch {}/{}: repos {}-{} ({})",
            batch_idx + 1,
            total_batches,
            chunk_start + 1,
            chunk_end,
            batch_names.join(", ")
        );

        if batch_idx == 0 {
            // First batch: get hero_title + bio + projects
            let prompt = build_llm_prompt_full(
                &body.github_username,
                &batch_contexts.to_vec(),
                &language,
                &batch_names,
            );
            eprintln!("[analyze] Batch 1 prompt size: {} bytes", prompt.len());

            match call_llm(&client, &api_url, &api_key, &model_name, &prompt, &language).await {
                Ok(r) => {
                    eprintln!("[analyze] Batch 1 OK: {} projects", r.projects.len());
                    hero_title = r.hero_title;
                    bio = r.bio;
                    all_llm_projects.extend(r.projects);
                }
                Err(e) => {
                    eprintln!("[analyze] ERROR - Batch 1 LLM: {}", e);
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("LLM error: {}", e)
                    }));
                }
            }
        } else {
            // Subsequent batches: projects only
            let prompt = build_llm_prompt_batch(
                &batch_contexts.to_vec(),
                &language,
                &batch_names,
            );
            eprintln!("[analyze] Batch {} prompt size: {} bytes", batch_idx + 1, prompt.len());

            match call_llm_batch(&client, &api_url, &api_key, &model_name, &prompt, &language).await {
                Ok(r) => {
                    eprintln!("[analyze] Batch {} OK: {} projects", batch_idx + 1, r.projects.len());
                    all_llm_projects.extend(r.projects);
                }
                Err(e) => {
                    eprintln!("[analyze] WARN - Batch {} failed: {}, continuing...", batch_idx + 1, e);
                    // Don't fail the whole request, just skip this batch
                }
            }
        }
    }

    eprintln!("[analyze] Total LLM projects: {}", all_llm_projects.len());

    // 5. Merge LLM results with repo data
    let project_cards: Vec<ProjectCard> = repos
        .iter()
        .map(|repo| {
            let llm_project = all_llm_projects
                .iter()
                .find(|p| p.name.to_lowercase() == repo.name.to_lowercase());

            ProjectCard {
                name: repo.name.clone(),
                problem_solved: llm_project
                    .map(|p| p.problem_solved.clone())
                    .unwrap_or_else(|| {
                        repo.description
                            .clone()
                            .unwrap_or_else(|| "No description available.".to_string())
                    }),
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
                stars: repo.stars,
                forks: repo.forks,
                html_url: repo.html_url.clone(),
                description: repo.description.clone(),
            }
        })
        .collect();

    let response = AnalyzeResponse {
        username: body.github_username.clone(),
        avatar_url: user.avatar_url,
        profile_url: user.html_url,
        hero_title,
        bio,
        projects: project_cards,
    };

    HttpResponse::Ok().json(response)
}

// ─── Main ───────────────────────────────────────────────────────────────────

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    println!("🚀 Git2Page server running at http://localhost:5001");

    HttpServer::new(|| {
        let json_cfg = web::JsonConfig::default()
            .limit(1048576)
            .error_handler(|err, _req| {
                let detail = err.to_string();
                eprintln!("[json_error] {}", detail);
                let response = HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Invalid request: {}", detail)
                }));
                actix_web::error::InternalError::from_response(err, response).into()
            });

        App::new()
            .app_data(json_cfg)
            .route("/config", web::get().to(get_config))
            .route("/analyze", web::post().to(analyze))
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("0.0.0.0:5001")?
    .run()
    .await
}
