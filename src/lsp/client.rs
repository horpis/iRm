// src/lsp/client.rs (упрощенно)
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub struct LspClient {
    server_in: tokio::process::ChildStdin,
    server_out: BufReader<tokio::process::ChildStdout>,
    request_id: i32,
}

impl LspClient {
    pub async fn start(command: &str) -> anyhow::Result<Self> {
        let mut child = Command::new(command)
            .arg("--stdio") // Пример аргумента для rust-analyzer
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        Ok(Self {
            server_in: child.stdin.take().unwrap(),
            server_out: BufReader::new(child.stdout.take().unwrap()),
            request_id: 0,
        })
    }

    pub async fn send_request(&mut self, method: &str, params: serde_json::Value) -> anyhow::Result<()> {
        self.request_id += 1;
        let rpc = serde_json::json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params
        });
        
        let msg = format!("Content-Length: {}\r\n\r\n{}", 
            serde_json::to_vec(&rpc)?.len(), 
            serde_json::to_string(&rpc)?
        );
        self.server_in.write_all(msg.as_bytes()).await?;
        Ok(())
    }
    
    // read_response() будет парсить заголовки Content-Length и тело JSON
}
